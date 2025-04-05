#![allow(dead_code)]

use crate::{
    internals::clock,
    interrupts::{self, Mode},
};
use libfenix::{
    clear_bit,
    gpio::{GpioBank, GpioPin},
    noop, read_addr, read_bit, set_bit, write_addr, GPIO1,
};

const GPIO_OE: u32 = 0x134;
const GPIO_DATAIN: u32 = 0x138;
const GPIO_DATAOUT: u32 = 0x13C;

const GPIO_IRQSTATUS_RAW_0: u32 = 0x24;
const GPIO_IRQSTATUS_0: u32 = 0x2C;
const GPIO_IRQSTATUS_SET0: u32 = 0x34;
const GPIO_RISINGDETECT: u32 = 0x148;
const GPIO_FALLINGDETECT: u32 = 0x14C;

const GPIOINT1A: u32 = 98;

pub fn initialize() {
    clock::enable(clock::FuncClock::Gpio1);

    interrupts::enable_interrupt(GPIOINT1A, Mode::IRQ, 1);
    interrupts::register_handler(handle_interrupts, GPIOINT1A as usize);

    for i in 21..=24 {
        pin_mode((i, GpioBank::Gpio1), GpioMode::Output);
    }
}

pub fn pin_mode((pin, bank): GpioPin, mode: GpioMode) {
    match mode {
        GpioMode::Input => {
            set_bit(bank as u32 + GPIO_OE, pin);
        }
        GpioMode::Output => {
            clear_bit(bank as u32 + GPIO_OE, pin);
        }
    }
}

pub fn write((pin, bank): GpioPin, value: bool) {
    if value {
        set_bit(bank as u32 + GPIO_DATAOUT, pin);
    } else {
        clear_bit(bank as u32 + GPIO_DATAOUT, pin);
    }
}

pub fn read((pin, bank): GpioPin) -> bool {
    read_bit(bank as u32 + GPIO_DATAIN, pin)
}

pub enum GpioMode {
    Input,
    Output,
}

static mut GPIO_INTERRUPT_HANDLERS: [fn(); 32] = [noop; 32];

// TODO, don't use just GPIO1 but make it dynamic
fn handle_interrupts() {
    let irq_raw = read_addr(GPIO1 + GPIO_IRQSTATUS_RAW_0);
    let number = irq_raw.trailing_zeros();

    unsafe {
        GPIO_INTERRUPT_HANDLERS[number as usize]();
    }

    write_addr(GPIO1 + GPIO_IRQSTATUS_0, 1 << number);
}

pub fn enable_interrupt(pin: u32, interrupt: GpioInterrupt, handler: fn()) {
    unsafe {
        GPIO_INTERRUPT_HANDLERS[pin as usize] = handler;
    }

    set_bit(GPIO1 + GPIO_IRQSTATUS_SET0, pin);

    match interrupt {
        GpioInterrupt::Rising => set_bit(GPIO1 + GPIO_RISINGDETECT, pin),
        GpioInterrupt::Falling => set_bit(GPIO1 + GPIO_FALLINGDETECT, pin),
        GpioInterrupt::Change => {
            set_bit(GPIO1 + GPIO_RISINGDETECT, pin);
            set_bit(GPIO1 + GPIO_FALLINGDETECT, pin)
        }
    }
}

pub enum GpioInterrupt {
    Rising,
    Falling,
    Change,
}
