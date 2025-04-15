#[cfg(feature = "bblack")]
use crate::boards::bblack::peripherals::gpio;
use crate::boards::bblack::peripherals::gpio::Register;

pub use gpio::{GpioBank, pins};

pub enum GpioMode {
    Input,
    Output,
}

pub type GpioPin = (u8, GpioBank);

pub trait GpioRegister {
    type Bank;

    fn init(&mut self);

    fn pin_mode(&mut self, pin: GpioPin, mode: GpioMode);

    fn write(&mut self, pin: GpioPin, value: bool);

    fn read(&self, pin: GpioPin) -> bool;
}

static mut REGISTER: Register = Register::new();

#[allow(static_mut_refs)]
pub(crate) fn init() {
    unsafe {
        REGISTER.init();
    }
}

#[allow(static_mut_refs)]
pub fn pin_mode(pin: GpioPin, mode: GpioMode) {
    unsafe {
        REGISTER.pin_mode(pin, mode);
    }
}

#[allow(static_mut_refs)]
pub fn write(pin: GpioPin, value: bool) {
    unsafe {
        REGISTER.write(pin, value);
    }
}

#[allow(static_mut_refs)]
pub fn read(pin: GpioPin) -> bool {
    unsafe { REGISTER.read(pin) }
}
