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

const XDR: u32 = 1 << 14; // Transmit Draining
const RDR: u32 = 1 << 13; // Receive Draining
const XRDY: u32 = 1 << 4; // Transmit Ready
const RRDY: u32 = 1 << 3; // Receive Ready
const NACK: u32 = 1 << 1; // No Acknowledge

const RECEIVE_THRESHOLD: u32 = 16;
const TRANSMIT_THRESHOLD: u32 = 16;

const TEST_ENABLE: u32 = 1 << 15;
const TEST_MODE: u32 = 12;

pub fn initialize() {
    let i2c = get_i2c();
    i2c.initialize();
    i2c.enable_test_mode();
}

pub fn transmit(data: &[u8]) {
    let i2c = get_i2c();
    i2c.transmit(data);
}

#[allow(static_mut_refs)]
fn get_i2c() -> &'static mut I2C {
    unsafe { &mut I2C }
}

static mut I2C: I2C = I2C::new(I2cModule::I2C2);

struct I2C {
    module: I2cModule,
    receive_buffer: Vec<u8>,
    transmit_buffer: Vec<u8>,
    transmit_index: usize,
}

impl I2C {
    const fn new(module: I2cModule) -> Self {
        Self {
            module,
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
        self.set_mode();
        self.setup_irq();
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

    fn set_mode(&self) {
        let value = read_addr(self.base() + I2C_CON);
        write_addr(self.base() + I2C_CON, value | 0x3 << 9); // setup master transmitter
    }

    fn setup_threshold(&self) {
        write_addr(
            self.base() + I2C_BUF,
            (RECEIVE_THRESHOLD - 1) << 8 | (TRANSMIT_THRESHOLD - 1),
        );
    }

    fn setup_irq(&self) {
        let value = read_addr(self.base() + I2C_IRQSTATUS_SET);
        write_addr(
            self.base() + I2C_IRQSTATUS_SET,
            value | XRDY | RRDY | XDR | RDR | NACK,
        );
    }

    fn enable_test_mode(&self) {
        let value = read_addr(self.base() + I2C_SYSTEST);
        write_addr(
            self.base() + I2C_SYSTEST,
            value | TEST_ENABLE | (0x3 << TEST_MODE),
        );
    }

    fn transmit(&mut self, data: &[u8]) {
        self.transmit_buffer.clear();
        for byte in data {
            self.transmit_buffer.push(*byte);
        }

        self.set_slave(0x50);
        self.set_count(data.len() as u32);
        while self.busy() {}
        self.set_start_stop();
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

        if value & XRDY != 0 {
            for _ in 0..min(TRANSMIT_THRESHOLD, self.transmit_buffer.len() as u32) {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, XRDY);
            return;
        }

        if value & XDR != 0 {
            for _ in 0..self.transmit_bytes_available() {
                self.write_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, XDR);
            return;
        }

        if value & RRDY != 0 {
            for _ in 0..RECEIVE_THRESHOLD {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, RRDY);
            return;
        }

        if value & RDR != 0 {
            for _ in 0..self.receive_bytes_available() {
                self.read_data();
            }

            write_addr(self.base() + I2C_IRQSTATUS, RDR);
            return;
        }

        if value & NACK != 0 {
            write_addr(self.base() + I2C_IRQSTATUS, NACK);
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
