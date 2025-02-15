#![no_std]
#![no_main]

use internals::timer::{self, millis, wait};
use interrupts::Interrupt;
use peripherals::gpio::{
    self,
    pins::{GPIO1_22, GPIO1_23, GPIO1_24, GPIO1_28},
    GpioBank, GpioMode,
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
    timer::initialize();

    for i in 21..=24 {
        gpio::pin_mode((i, GpioBank::Gpio1), GpioMode::Output);
    }

    gpio::pin_mode(GPIO1_28, GpioMode::Input);

    gpio::write(GPIO1_24, true);

    let start = millis();
    loop {
        if millis() - start >= 1{
            gpio::write(GPIO1_23, true);
        }

        gpio::write(GPIO1_22, gpio::read(GPIO1_28));
    }
}

#[no_mangle]
fn handle_interrupt() {
    let interrupt = Interrupt::get_current();
    interrupt.execute();
    interrupt.clear();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
