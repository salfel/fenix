#![allow(dead_code)]

use crate::{
    interrupts::{self, register_handler, Mode},
    sys::{clear_bit, noop, read_addr, read_bit, set_bit, write_addr, CM_PER, GPIO1},
};

const CM_PER_GPIO1_CLKCTRL: u32 = 0xAC;

const GPIO_OE: u32 = 0x134;
const GPIO_DATAIN: u32 = 0x138;
const GPIO_DATAOUT: u32 = 0x13C;

const GPIO_IRQSTATUS_RAW_0: u32 = 0x24;
const GPIO_IRQSTATUS_0: u32 = 0x2C;
const GPIO_IRQSTATUS_SET0: u32 = 0x34;
const GPIO_RISINGDETECT: u32 = 0x148;
const GPIO_FALLINGDETECT: u32 = 0x14C;

const GPIOINT1A: u32 = 98;

type GpioPin = (u32, GpioBank);

pub fn initialize() {
    write_addr(CM_PER + CM_PER_GPIO1_CLKCTRL, 2);
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

pub enum GpioBank {
    Gpio0 = 0x44E0_7000,
    Gpio1 = 0x4804_C000,
    Gpio2 = 0x481A_C000,
    Gpio3 = 0x481A_E000,
}

pub enum GpioMode {
    Input,
    Output,
}

static mut GPIO_INTERRUPT_HANDLERS: [fn(); 32] = [noop; 32];

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

pub mod pins {
    use super::{GpioBank, GpioPin};

    pub const GPIO0_0: GpioPin = (0, GpioBank::Gpio0);
    pub const GPIO0_1: GpioPin = (1, GpioBank::Gpio0);
    pub const GPIO0_2: GpioPin = (2, GpioBank::Gpio0);
    pub const GPIO0_3: GpioPin = (3, GpioBank::Gpio0);
    pub const GPIO0_4: GpioPin = (4, GpioBank::Gpio0);
    pub const GPIO0_5: GpioPin = (5, GpioBank::Gpio0);
    pub const GPIO0_6: GpioPin = (6, GpioBank::Gpio0);
    pub const GPIO0_7: GpioPin = (7, GpioBank::Gpio0);
    pub const GPIO0_8: GpioPin = (8, GpioBank::Gpio0);
    pub const GPIO0_9: GpioPin = (9, GpioBank::Gpio0);
    pub const GPIO0_10: GpioPin = (10, GpioBank::Gpio0);
    pub const GPIO0_11: GpioPin = (11, GpioBank::Gpio0);
    pub const GPIO0_12: GpioPin = (12, GpioBank::Gpio0);
    pub const GPIO0_13: GpioPin = (13, GpioBank::Gpio0);
    pub const GPIO0_14: GpioPin = (14, GpioBank::Gpio0);
    pub const GPIO0_15: GpioPin = (15, GpioBank::Gpio0);
    pub const GPIO0_16: GpioPin = (16, GpioBank::Gpio0);
    pub const GPIO0_17: GpioPin = (17, GpioBank::Gpio0);
    pub const GPIO0_18: GpioPin = (18, GpioBank::Gpio0);
    pub const GPIO0_19: GpioPin = (19, GpioBank::Gpio0);
    pub const GPIO0_20: GpioPin = (20, GpioBank::Gpio0);
    pub const GPIO0_21: GpioPin = (21, GpioBank::Gpio0);
    pub const GPIO0_22: GpioPin = (22, GpioBank::Gpio0);
    pub const GPIO0_23: GpioPin = (23, GpioBank::Gpio0);
    pub const GPIO0_24: GpioPin = (24, GpioBank::Gpio0);
    pub const GPIO0_25: GpioPin = (25, GpioBank::Gpio0);
    pub const GPIO0_26: GpioPin = (26, GpioBank::Gpio0);
    pub const GPIO0_27: GpioPin = (27, GpioBank::Gpio0);
    pub const GPIO0_28: GpioPin = (28, GpioBank::Gpio0);
    pub const GPIO0_29: GpioPin = (29, GpioBank::Gpio0);
    pub const GPIO0_30: GpioPin = (30, GpioBank::Gpio0);
    pub const GPIO0_31: GpioPin = (31, GpioBank::Gpio0);

    pub const GPIO1_0: GpioPin = (0, GpioBank::Gpio1);
    pub const GPIO1_1: GpioPin = (1, GpioBank::Gpio1);
    pub const GPIO1_2: GpioPin = (2, GpioBank::Gpio1);
    pub const GPIO1_3: GpioPin = (3, GpioBank::Gpio1);
    pub const GPIO1_4: GpioPin = (4, GpioBank::Gpio1);
    pub const GPIO1_5: GpioPin = (5, GpioBank::Gpio1);
    pub const GPIO1_6: GpioPin = (6, GpioBank::Gpio1);
    pub const GPIO1_7: GpioPin = (7, GpioBank::Gpio1);
    pub const GPIO1_8: GpioPin = (8, GpioBank::Gpio1);
    pub const GPIO1_9: GpioPin = (9, GpioBank::Gpio1);
    pub const GPIO1_10: GpioPin = (10, GpioBank::Gpio1);
    pub const GPIO1_11: GpioPin = (11, GpioBank::Gpio1);
    pub const GPIO1_12: GpioPin = (12, GpioBank::Gpio1);
    pub const GPIO1_13: GpioPin = (13, GpioBank::Gpio1);
    pub const GPIO1_14: GpioPin = (14, GpioBank::Gpio1);
    pub const GPIO1_15: GpioPin = (15, GpioBank::Gpio1);
    pub const GPIO1_16: GpioPin = (16, GpioBank::Gpio1);
    pub const GPIO1_17: GpioPin = (17, GpioBank::Gpio1);
    pub const GPIO1_18: GpioPin = (18, GpioBank::Gpio1);
    pub const GPIO1_19: GpioPin = (19, GpioBank::Gpio1);
    pub const GPIO1_20: GpioPin = (20, GpioBank::Gpio1);
    pub const GPIO1_21: GpioPin = (21, GpioBank::Gpio1);
    pub const GPIO1_22: GpioPin = (22, GpioBank::Gpio1);
    pub const GPIO1_23: GpioPin = (23, GpioBank::Gpio1);
    pub const GPIO1_24: GpioPin = (24, GpioBank::Gpio1);
    pub const GPIO1_25: GpioPin = (25, GpioBank::Gpio1);
    pub const GPIO1_26: GpioPin = (26, GpioBank::Gpio1);
    pub const GPIO1_27: GpioPin = (27, GpioBank::Gpio1);
    pub const GPIO1_28: GpioPin = (28, GpioBank::Gpio1);
    pub const GPIO1_29: GpioPin = (29, GpioBank::Gpio1);
    pub const GPIO1_30: GpioPin = (30, GpioBank::Gpio1);
    pub const GPIO1_31: GpioPin = (31, GpioBank::Gpio1);

    pub const GPIO2_0: GpioPin = (0, GpioBank::Gpio2);
    pub const GPIO2_1: GpioPin = (1, GpioBank::Gpio2);
    pub const GPIO2_2: GpioPin = (2, GpioBank::Gpio2);
    pub const GPIO2_3: GpioPin = (3, GpioBank::Gpio2);
    pub const GPIO2_4: GpioPin = (4, GpioBank::Gpio2);
    pub const GPIO2_5: GpioPin = (5, GpioBank::Gpio2);
    pub const GPIO2_6: GpioPin = (6, GpioBank::Gpio2);
    pub const GPIO2_7: GpioPin = (7, GpioBank::Gpio2);
    pub const GPIO2_8: GpioPin = (8, GpioBank::Gpio2);
    pub const GPIO2_9: GpioPin = (9, GpioBank::Gpio2);
    pub const GPIO2_10: GpioPin = (10, GpioBank::Gpio2);
    pub const GPIO2_11: GpioPin = (11, GpioBank::Gpio2);
    pub const GPIO2_12: GpioPin = (12, GpioBank::Gpio2);
    pub const GPIO2_13: GpioPin = (13, GpioBank::Gpio2);
    pub const GPIO2_14: GpioPin = (14, GpioBank::Gpio2);
    pub const GPIO2_15: GpioPin = (15, GpioBank::Gpio2);
    pub const GPIO2_16: GpioPin = (16, GpioBank::Gpio2);
    pub const GPIO2_17: GpioPin = (17, GpioBank::Gpio2);
    pub const GPIO2_18: GpioPin = (18, GpioBank::Gpio2);
    pub const GPIO2_19: GpioPin = (19, GpioBank::Gpio2);
    pub const GPIO2_20: GpioPin = (20, GpioBank::Gpio2);
    pub const GPIO2_21: GpioPin = (21, GpioBank::Gpio2);
    pub const GPIO2_22: GpioPin = (22, GpioBank::Gpio2);
    pub const GPIO2_23: GpioPin = (23, GpioBank::Gpio2);
    pub const GPIO2_24: GpioPin = (24, GpioBank::Gpio2);
    pub const GPIO2_25: GpioPin = (25, GpioBank::Gpio2);
    pub const GPIO2_26: GpioPin = (26, GpioBank::Gpio2);
    pub const GPIO2_27: GpioPin = (27, GpioBank::Gpio2);
    pub const GPIO2_28: GpioPin = (28, GpioBank::Gpio2);
    pub const GPIO2_29: GpioPin = (29, GpioBank::Gpio2);
    pub const GPIO2_30: GpioPin = (30, GpioBank::Gpio2);
    pub const GPIO2_31: GpioPin = (31, GpioBank::Gpio2);

    pub const GPIO3_0: GpioPin = (0, GpioBank::Gpio3);
    pub const GPIO3_1: GpioPin = (1, GpioBank::Gpio3);
    pub const GPIO3_2: GpioPin = (2, GpioBank::Gpio3);
    pub const GPIO3_3: GpioPin = (3, GpioBank::Gpio3);
    pub const GPIO3_4: GpioPin = (4, GpioBank::Gpio3);
    pub const GPIO3_5: GpioPin = (5, GpioBank::Gpio3);
    pub const GPIO3_6: GpioPin = (6, GpioBank::Gpio3);
    pub const GPIO3_7: GpioPin = (7, GpioBank::Gpio3);
    pub const GPIO3_8: GpioPin = (8, GpioBank::Gpio3);
    pub const GPIO3_9: GpioPin = (9, GpioBank::Gpio3);
    pub const GPIO3_10: GpioPin = (10, GpioBank::Gpio3);
    pub const GPIO3_11: GpioPin = (11, GpioBank::Gpio3);
    pub const GPIO3_12: GpioPin = (12, GpioBank::Gpio3);
    pub const GPIO3_13: GpioPin = (13, GpioBank::Gpio3);
    pub const GPIO3_14: GpioPin = (14, GpioBank::Gpio3);
    pub const GPIO3_15: GpioPin = (15, GpioBank::Gpio3);
    pub const GPIO3_16: GpioPin = (16, GpioBank::Gpio3);
    pub const GPIO3_17: GpioPin = (17, GpioBank::Gpio3);
    pub const GPIO3_18: GpioPin = (18, GpioBank::Gpio3);
    pub const GPIO3_19: GpioPin = (19, GpioBank::Gpio3);
    pub const GPIO3_20: GpioPin = (20, GpioBank::Gpio3);
    pub const GPIO3_21: GpioPin = (21, GpioBank::Gpio3);
    pub const GPIO3_22: GpioPin = (22, GpioBank::Gpio3);
    pub const GPIO3_23: GpioPin = (23, GpioBank::Gpio3);
    pub const GPIO3_24: GpioPin = (24, GpioBank::Gpio3);
    pub const GPIO3_25: GpioPin = (25, GpioBank::Gpio3);
    pub const GPIO3_26: GpioPin = (26, GpioBank::Gpio3);
    pub const GPIO3_27: GpioPin = (27, GpioBank::Gpio3);
    pub const GPIO3_28: GpioPin = (28, GpioBank::Gpio3);
    pub const GPIO3_29: GpioPin = (29, GpioBank::Gpio3);
    pub const GPIO3_30: GpioPin = (30, GpioBank::Gpio3);
    pub const GPIO3_31: GpioPin = (31, GpioBank::Gpio3);
}
