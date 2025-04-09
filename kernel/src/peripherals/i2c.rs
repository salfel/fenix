use core::cmp::min;

use crate::{
    alloc::vec::Vec,
    internals::clock::{self, FuncClock},
    interrupts::{self, Interrupt, Mode},
};
use libfenix::{self, read_addr, set_bit, write_addr};

const SYS_CLOCK: u32 = 48_000_000;
const INTERNAL_CLOCK: u32 = 12_000_000;
const OUTPUT_CLOCK: u32 = 100_000;

const I2C_SYSC: u32 = 0x10;
const I2C_IRQSTATUS_RAW: u32 = 0x24;
const I2C_IRQSTATUS: u32 = 0x28;
const I2C_IRQSTATUS_SET: u32 = 0x2C;
const I2C_IRQSTATUS_CLR: u32 = 0x30;
const I2C_CNT: u32 = 0x98;
const I2C_DATA: u32 = 0x9C;
const I2C_CON: u32 = 0xA4;
const I2C_OA: u32 = 0xA8;
const I2C_SA: u32 = 0xAC;
const I2C_PSC: u32 = 0xB0;
const I2C_SCLL: u32 = 0xB4;
const I2C_SCLH: u32 = 0xB8;
const I2C_SYSTEST: u32 = 0xBC;
const I2C_SYSS: u32 = 0x90;
const I2C_BUF: u32 = 0x94;
const I2C_BUFSTAT: u32 = 0xC0;

const RECEIVE_THRESHOLD: u32 = 16;
const TRANSMIT_THRESHOLD: u32 = 16;

const TEST_ENABLE: u32 = 1 << 15;
const TEST_MODE: u32 = 12;

pub fn initialize() {
    let i2c = get_i2c();
    i2c.initialize();
}

#[allow(static_mut_refs)]
pub fn get_i2c() -> &'static mut I2C {
    unsafe { &mut I2C }
}

static mut I2C: I2C = I2C::new(I2cModule::I2C2);

pub struct I2C {
    module: I2cModule,
    mode: Option<I2cMode>,
    address: Option<u32>,
    receive_buffer: Vec<u8>,
    transmit_buffer: Vec<u8>,
    transmit_index: usize,
}

impl I2C {
    const fn new(module: I2cModule) -> Self {
        Self {
            module,
            mode: None,
            address: None,
            receive_buffer: Vec::new(),
            transmit_buffer: Vec::new(),
            transmit_index: 0,
        }
    }

    fn base(&self) -> u32 {
        self.module as u32
    }

    fn initialize(&self) {
        clock::enable(FuncClock::I2C2);

        interrupts::enable_interrupt(Interrupt::I2C2INT, Mode::IRQ, 2); // enable irq
        interrupts::register_handler(irq_handler, Interrupt::I2C2INT); // register handler

        // config
        self.soft_reset();
        self.init_clocks();
        self.set_own_address();
        self.enable();
        self.wait_reset();

        // init
        self.setup_threshold();
    }

    fn soft_reset(&self) {
        write_addr(
            self.base() + I2C_SYSC,
            read_addr(self.base() + I2C_SYSC) | 0x2,
        );
    }

    fn init_clocks(&self) {
        let prescaler = (SYS_CLOCK / INTERNAL_CLOCK) - 1;
        write_addr(self.base() + I2C_PSC, prescaler);

        let mut divider = INTERNAL_CLOCK / OUTPUT_CLOCK;
        divider /= 2;

        write_addr(self.base() + I2C_SCLL, divider - 7);
        write_addr(self.base() + I2C_SCLH, divider - 5);
    }

    fn set_own_address(&self) {
        write_addr(self.base() + I2C_OA, 0x50);
    }

    fn enable(&self) {
        set_bit(self.base() + I2C_CON, 15);
    }

    fn wait_reset(&self) {
        while read_addr(self.base() + I2C_SYSS) & 0x1 == 0 {}
    }

    fn set_mode(&self, mode: &I2cMode) {
        let value = read_addr(self.base() + I2C_CON);
        let is_transmitter = matches!(mode, I2cMode::Transmitter);
        write_addr(
            self.base() + I2C_CON,
            value | 1 << 10 | (is_transmitter as u32) << 9,
        );
    }

    fn setup_threshold(&self) {
        write_addr(
            self.base() + I2C_BUF,
            (RECEIVE_THRESHOLD - 1) << 8 | (TRANSMIT_THRESHOLD - 1),
        );
    }

    fn enable_irq(&self, irq: I2cInterrupt) {
        let value = read_addr(self.base() + I2C_IRQSTATUS_SET);
        write_addr(self.base() + I2C_IRQSTATUS_SET, value | irq as u32);
    }

    fn disable_irq(&self, irq: I2cInterrupt) {
        let value = read_addr(self.base() + I2C_IRQSTATUS_CLR);
        write_addr(self.base() + I2C_IRQSTATUS_CLR, value | irq as u32);
    }

    #[allow(unused)]
    fn enable_test_mode(&self) {
        let value = read_addr(self.base() + I2C_SYSTEST);
        write_addr(
            self.base() + I2C_SYSTEST,
            value | TEST_ENABLE | (0x3 << TEST_MODE),
        );
    }

    pub fn begin(&mut self, slave_address: u32) {
        self.set_slave(slave_address);
        self.address = Some(slave_address);

        let mode = I2cMode::Transmitter;
        self.set_mode(&mode);
        self.mode = Some(mode);

        self.transmit_buffer.clear();

        self.enable_interrupts();

        while self.busy() {}
    }

    pub fn transmit(&mut self, data: &[u8]) {
        if self.address.is_none() {
            panic!("I2C not initialized");
        }

        for byte in data {
            self.transmit_buffer.push(*byte);
        }

        self.set_count(data.len() as u32);
        self.set_start_stop();
    }

    pub fn end(&mut self) {
        self.address = None;
        self.mode = None;
        self.disable_interrupts();
    }

    fn enable_interrupts(&self) {
        if let Some(mode) = &self.mode {
            for interrupt in mode.interrupts() {
                self.enable_irq(*interrupt);
            }
        }
    }

    fn disable_interrupts(&self) {
        if let Some(mode) = &self.mode {
            for interrupt in mode.interrupts() {
                self.disable_irq(*interrupt);
            }
        }
    }

    fn set_slave(&self, address: u32) {
        write_addr(self.base() + I2C_SA, address);
    }

    fn set_count(&self, count: u32) {
        write_addr(self.base() + I2C_CNT, count);
    }

    fn busy(&self) -> bool {
        let value = read_addr(self.base() + I2C_IRQSTATUS_RAW);
        value & (1 << 12) != 0
    }

    fn set_start_stop(&self) {
        let value = read_addr(self.base() + I2C_CON);
        write_addr(self.base() + I2C_CON, value | 0x3);
    }

    fn start(&self) {
        let value = read_addr(self.base() + I2C_CON);
        write_addr(self.base() + I2C_CON, value | 0x1);
    }

    fn write_data(&mut self) {
        let data = self.transmit_buffer.get(self.transmit_index).unwrap();
        write_addr(self.base() + I2C_DATA, *data as u32);
        self.transmit_index += 1;
    }

    fn read_data(&mut self) {
        let data = read_addr(self.base() + I2C_DATA) as u8;
        self.receive_buffer.push(data);
    }

    fn transmit_bytes_available(&self) -> u32 {
        read_addr(self.base() + I2C_BUFSTAT) & 0x3F
    }

    fn receive_bytes_available(&self) -> u32 {
        (read_addr(self.base() + I2C_BUFSTAT) >> 8) & 0x3F
    }

    fn irq_handler(&mut self) {
        let value = read_addr(self.base() + I2C_IRQSTATUS);

        if value & I2cInterrupt::XRDY as u32 != 0 {
            for _ in 0..min(TRANSMIT_THRESHOLD, self.transmit_buffer.len() as u32) {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::XRDY as u32);
            return;
        }

        if value & I2cInterrupt::XDR as u32 != 0 {
            for _ in 0..self.transmit_bytes_available() {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::XDR as u32);
            return;
        }

        if value & I2cInterrupt::RRDY as u32 != 0 {
            for _ in 0..RECEIVE_THRESHOLD {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::RRDY as u32);
            return;
        }

        if value & I2cInterrupt::RDR as u32 != 0 {
            for _ in 0..self.receive_bytes_available() {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::RDR as u32);
            return;
        }

        if value & I2cInterrupt::ARDY as u32 != 0 {
            self.end();

            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::ARDY as u32);
            return;
        }

        if value & I2cInterrupt::NACK as u32 != 0 {
            write_addr(self.base() + I2C_IRQSTATUS, I2cInterrupt::NACK as u32);
        }
    }
}

fn irq_handler() {
    let i2c = get_i2c();
    i2c.irq_handler()
}

#[derive(Clone, Copy)]
enum I2cModule {
    I2C2 = 0x4819_C000,
}

#[allow(unused)]
enum I2cMode {
    Transmitter,
    Receiver,
}

impl I2cMode {
    fn interrupts(&self) -> &[I2cInterrupt] {
        match self {
            I2cMode::Transmitter => &[
                I2cInterrupt::XRDY,
                I2cInterrupt::XDR,
                I2cInterrupt::ARDY,
                I2cInterrupt::NACK,
            ],
            I2cMode::Receiver => &[
                I2cInterrupt::RRDY,
                I2cInterrupt::RDR,
                I2cInterrupt::ARDY,
                I2cInterrupt::NACK,
            ],
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
enum I2cInterrupt {
    XDR = 1 << 14, // Transmit Draining
    RDR = 1 << 13, // Receive Draining
    XRDY = 1 << 4, // Transmit Ready
    RRDY = 1 << 3, // Receive Ready
    ARDY = 1 << 2, // Access Ready
    NACK = 1 << 1, // No Acknowledge
}
