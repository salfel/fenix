use crate::utils::{clear_bit, read_bit, set_bit, write_addr};

const CM_PER: u32 = 0x44E00000;
const CM_PER_GPIO1_CLKCTRL: u32 = 0xAC;

const GPIO1: u32 = 0x4804C000;
const GPIO_OE: u32 = 0x134;
const GPIO_DATAIN: u32 = 0x138;
const GPIO_DATAOUT: u32 = 0x13C;

pub fn initialize() {
    write_addr(CM_PER + CM_PER_GPIO1_CLKCTRL, 2);
}

pub fn pin_mode(pin: u32, mode: GpioMode) {
    match mode {
        GpioMode::Input => {
            set_bit(GPIO1 + GPIO_OE, pin);
        }
        GpioMode::Output => {
            clear_bit(GPIO1 + GPIO_OE, pin);
        }
    }
}

pub fn write(pin: u32, value: bool) {
    if value {
        set_bit(GPIO1 + GPIO_DATAOUT, pin);
    } else {
        clear_bit(GPIO1 + GPIO_DATAOUT, pin);
    }
}

pub fn read(pin: u32) -> bool {
    read_bit(GPIO1 + GPIO_DATAIN, pin)
}

pub enum GpioMode {
    Input,
    Output,
}
