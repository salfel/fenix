use crate::{
    internals::clock::{self, FuncClock},
    pinmux,
};
use libfenix::{read_addr, set_bit, write_addr};

const I2C2: u32 = 0x4819C000;

const SYS_CLOCK: u32 = 48_000_000;
const INTERNAL_CLOCK: u32 = 12_000_000;

const I2C_BASE: u32 = I2C2;

const I2C_CON: u32 = 0xA4;
const I2C_OA: u32 = 0xA8;
const I2C_PSC: u32 = 0xB0;
const I2C_SCLL: u32 = 0xB4;
const I2C_SCLH: u32 = 0xB8;

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

fn setup_irq() {
    // TODD setup irq
}
