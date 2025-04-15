#[cfg(feature = "bblack")]
use crate::boards::bblack::peripherals::gpio;
use crate::boards::bblack::peripherals::gpio::Register;

pub use gpio::{pins, GpioBank};

pub enum GpioMode {
    Input,
    Output,
}

pub type GpioPin = (u8, GpioBank);

pub trait GpioRegister {
    type Bank;

    fn init(&self);

    fn pin_mode(&self, pin: GpioPin, mode: GpioMode);

    fn write(&self, pin: GpioPin, value: bool);

    fn read(&self, pin: GpioPin) -> bool;
}

static GPIO_REGISTER: Register = Register::new();

pub(crate) fn init() {
    GPIO_REGISTER.init();
}

pub fn pin_mode(pin: GpioPin, mode: GpioMode) {
    GPIO_REGISTER.pin_mode(pin, mode);
}

pub fn write(pin: GpioPin, value: bool) {
    GPIO_REGISTER.write(pin, value);
}

pub fn read(pin: GpioPin) -> bool {
    GPIO_REGISTER.read(pin)
}
