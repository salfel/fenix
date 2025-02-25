#![no_std]
#![no_main]

use internals::sysclock;
use peripherals::{
    gpio::{
        self,
        pins::{GPIO1_22, GPIO1_23, GPIO1_24, GPIO1_28},
        GpioBank, GpioMode,
    },
    i2c::{self, I2CModule},
};

pub mod internals;
pub mod interrupts;
pub mod peripherals;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn main() {
    pinmux::configure();
    gpio::initialize();
    sysclock::initialize();

    for i in 21..=24 {
        gpio::pin_mode((i, GpioBank::Gpio1), GpioMode::Output);
    }

    i2c::register_i2c_module(I2CModule::I2C1, i2c::Mode::MasterTransmit);
    let i2c1 = i2c::get_i2c_module(I2CModule::I2C1).unwrap();

    gpio::write(GPIO1_24, true);

    i2c1.begin();

    gpio::write(GPIO1_23, true);

    loop {
        gpio::write(GPIO1_22, gpio::read(GPIO1_28));
    }
}

#[no_mangle]
fn handle_interrupt() {
    let interrupt = interrupts::current();
    interrupts::execute(interrupt);
    interrupts::clear();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
