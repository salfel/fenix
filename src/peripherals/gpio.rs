#[cfg(feature = "bblack")]
use crate::boards::bblack::peripherals::gpio::{self, Register};

pub use gpio::{GpioBank, pins};

pub enum GpioMode {
    Input,
    Output,
}

pub type GpioPin = (u8, GpioBank);

pub(crate) trait GpioRegister {
    type Bank;

    fn init(&mut self);

    fn pin_mode(&mut self, pin: u8, bank: Self::Bank, mode: GpioMode);

    fn write(&mut self, pin: u8, bank: Self::Bank, value: bool);

    fn read(&self, pin: u8, bank: Self::Bank) -> bool;
}

static mut REGISTER: Register = Register::new();

#[allow(static_mut_refs)]
pub(crate) fn init() {
    unsafe {
        REGISTER.init();
    }
}

#[allow(static_mut_refs)]
pub fn pin_mode((pin, bank): GpioPin, mode: GpioMode) {
    unsafe {
        REGISTER.pin_mode(pin, bank, mode);
    }
}

#[allow(static_mut_refs)]
pub fn write((pin, bank): GpioPin, value: bool) {
    unsafe {
        REGISTER.write(pin, bank, value);
    }
}

#[allow(static_mut_refs)]
pub fn read((pin, bank): GpioPin) -> bool {
    unsafe { REGISTER.read(pin, bank) }
}
