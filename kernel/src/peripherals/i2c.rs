use crate::{
    internals::clock::{self, FuncClock},
    interrupts::{self, Interrupt, Mode},
};
use libfenix::{
    self,
    gpio::pins::{GPIO1_21, GPIO1_22},
    read_addr, set_bit, write_addr,
};

use super::gpio;

const I2C2: u32 = 0x4819C000;

const SYS_CLOCK: u32 = 48_000_000;
const INTERNAL_CLOCK: u32 = 12_000_000;
const OUTPUT_CLOCK: u32 = 100_000;

const I2C_BASE: u32 = I2C2;

const I2C_SYSC: u32 = 0x10;
const I2C_IRQSTATUS_RAW: u32 = 0x24;
const I2C_IRQSTATUS: u32 = 0x28;
const I2C_IRQSTATUS_SET: u32 = 0x2C;
const I2C_IRQSTATUS_CLR: u32 = 0x30;
const I2C_CNT: u32 = 0x98;
const I2C_DATA: u32 = 0x98;
const I2C_CON: u32 = 0xA4;
const I2C_OA: u32 = 0xA8;
const I2C_SA: u32 = 0xAC;
const I2C_PSC: u32 = 0xB0;
const I2C_SCLL: u32 = 0xB4;
const I2C_SCLH: u32 = 0xB8;
const I2C_SYSTEST: u32 = 0xBC;
const I2C_SYSS: u32 = 0x90;

const XRDY: u32 = 1 << 4;
const BF: u32 = 1 << 8;

const TEST_ENABLE: u32 = 1 << 15;
const TEST_MODE: u32 = 13;

pub fn initialize() {
    clock::enable(FuncClock::I2C2);

    interrupts::enable_interrupt(Interrupt::I2C2INT, Mode::IRQ, 2); // enable irq
    interrupts::register_handler(irq_handler, Interrupt::I2C2INT); // register handler

    // config
    soft_reset();
    init_clocks();
    set_own_address();
    enable();
    wait_reset();

    // init
    set_mode();
    setup_irq();
}

static mut BUS_FREE: bool = true;

pub fn irq_handler() {
    let value = read_addr(I2C_BASE + I2C_IRQSTATUS);

    if value & XRDY != 0 {
        write_addr(I2C_BASE + I2C_DATA, 0xFF);

        write_addr(I2C_BASE + I2C_IRQSTATUS, XRDY);
        gpio::write(GPIO1_21, true);
    }

    if value & BF != 0 {
        unsafe { BUS_FREE = true };

        write_addr(I2C_BASE + I2C_IRQSTATUS, BF);
        gpio::write(GPIO1_22, true);
    }
}

fn soft_reset() {
    write_addr(I2C_BASE + I2C_SYSC, read_addr(I2C_BASE + I2C_SYSC) | 0x2);
}

fn wait_reset() {
    while read_addr(I2C_BASE + I2C_SYSS) & 0x1 == 0 {}
}

fn init_clocks() {
    let prescaler = (SYS_CLOCK / INTERNAL_CLOCK) - 1;
    write_addr(I2C_BASE + I2C_PSC, prescaler);

    let mut divider = INTERNAL_CLOCK / OUTPUT_CLOCK;
    divider /= 2;

    write_addr(I2C_BASE + I2C_SCLL, divider - 7);
    write_addr(I2C_BASE + I2C_SCLH, divider - 5);
}

fn set_own_address() {
    write_addr(I2C_BASE + I2C_OA, 0x50);
}

fn enable() {
    set_bit(I2C_BASE + I2C_CON, 15);
}

fn set_mode() {
    let value = read_addr(I2C_BASE + I2C_CON);
    write_addr(I2C_BASE + I2C_CON, value | 0x3 << 9); // setup master transmitter
}

fn setup_irq() {
    let value = read_addr(I2C_BASE + I2C_IRQSTATUS_SET);
    write_addr(I2C_BASE + I2C_IRQSTATUS_SET, value | XRDY | BF);
}

fn set_slave(address: u32) {
    write_addr(I2C_BASE + I2C_SA, address);
}

fn set_count(count: u32) {
    write_addr(I2C_BASE + I2C_CNT, count);
}

fn busy() -> bool {
    let result = unsafe { !BUS_FREE };
    unsafe { BUS_FREE = false };
    result
}

fn set_start_stop() {
    let value = read_addr(I2C_BASE + I2C_CON);
    write_addr(I2C_BASE + I2C_CON, value | 0x3);
}

pub fn enable_test_mode() {
    let value = read_addr(I2C_BASE + I2C_SYSTEST);
    write_addr(
        I2C_BASE + I2C_SYSTEST,
        value | TEST_ENABLE | (0x3 << TEST_MODE),
    );
}

pub fn transmit() {
    set_slave(0x30);
    set_count(1);
    while busy() {}
    set_start_stop();
}
