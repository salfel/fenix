#![no_std]
#![no_main]

use peripherals::gpio::{
    self,
    pins::GPIO1_24,
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

    for i in 21..=24 {
        gpio::pin_mode((i, GpioBank::Gpio1), GpioMode::Output);
    }

    // Set the on gpio pin
    gpio::write(GPIO1_24, true);

    loop {}
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

#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr0() {}
