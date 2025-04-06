use crate::{
    internals::clock::{self, FuncClock},
    interrupts::{self, Interrupt, Mode},
};
use libfenix::{read_addr, set_bit, write_addr};

const I2C2: u32 = 0x4819C000;

const SYS_CLOCK: u32 = 48_000_000;
const INTERNAL_CLOCK: u32 = 12_000_000;

const I2C_BASE: u32 = I2C2;

const I2C_IRQSTATUS_RAW: u32 = 0x24;
const I2C_IRQSTATUS: u32 = 0x28;
const I2C_IRQSTATUS_SET: u32 = 0x2C;
const I2C_CNT: u32 = 0x98;
const I2C_DATA: u32 = 0x98;
const I2C_CON: u32 = 0xA4;
const I2C_OA: u32 = 0xA8;
const I2C_SA: u32 = 0xAC;
const I2C_PSC: u32 = 0xB0;
const I2C_SCLL: u32 = 0xB4;
const I2C_SCLH: u32 = 0xB8;
const I2C_SYSTEST: u32 = 0xBC;

pub fn initialize() {
    clock::enable(FuncClock::I2C2);

    init_clocks();
    set_own_address(0);
    enable();
    setup_mode();
    setup_irq();
}

fn init_clocks() {
    let mut divider = SYS_CLOCK / INTERNAL_CLOCK;
    write_addr(I2C_BASE + I2C_PSC, divider - 1);
    divider /= 2;

    write_addr(I2C_BASE + I2C_SCLL, divider - 7);
    write_addr(I2C_BASE + I2C_SCLH, divider - 5);
}

fn set_own_address(address: u32) {
    write_addr(I2C_BASE + I2C_OA, address);
}

fn enable() {
    set_bit(I2C_BASE + I2C_CON, 15);
}

fn setup_mode() {
    let value = read_addr(I2C_BASE + I2C_CON);
    write_addr(I2C_BASE + I2C_CON, value | 0x3 << 9);
}

static mut DATA_READY: bool = false;

fn setup_irq() {
    //interrupts::enable_interrupt(Interrupt::I2C2INT, Mode::IRQ, 1);
    //interrupts::register_handler(handle_irq, Interrupt::I2C2INT);
    //
    //set_bit(I2C_BASE + I2C_IRQSTATUS_SET, 4);
}

fn handle_irq() {
    unsafe {
        DATA_READY = true;
    }
}

fn busy() -> bool {
    let value = read_addr(I2C_BASE + I2C_IRQSTATUS_RAW);
    value & (1 << 12) != 0
}

fn wait_busy() {
    while busy() {}
}

fn start() {
    write_addr(I2C_BASE + I2C_CON, read_addr(I2C_BASE + I2C_CON) | 0x1);
    write_addr(
        I2C_BASE + I2C_CON,
        read_addr(I2C_BASE + I2C_CON) | (0x1 << 1),
    );
}

fn set_count(count: u32) {
    write_addr(I2C_BASE + I2C_CNT, count);
}

fn set_slave_address(address: u8) {
    write_addr(I2C_BASE + I2C_SA, address as u32);
}

fn trasmit_ready() -> bool {
    let value = read_addr(I2C_BASE + I2C_IRQSTATUS_RAW);
    value & (1 << 4) != 0

    //unsafe { DATA_READY }
}

pub fn transmit(data: u8, slave_address: u8) {
    set_count(1);
    set_slave_address(slave_address);
    wait_busy();
    start();
    if trasmit_ready() {
        write_addr(I2C_BASE + I2C_DATA, data as u32);
    }
}

pub fn enable_test_mode() {
    let mut value = 0;
    value |= 0x1 << 15; // enable module
    value |= 0x3 << 12; // loop back mode

    write_addr(I2C_BASE + I2C_SYSTEST, value);
}
