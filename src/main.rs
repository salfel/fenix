#![no_std]
#![no_main]

use peripherals::gpio::{self, pins::{GPIO1_22, GPIO1_24, GPIO1_28}, GpioBank, GpioMode};
use interrupts::Interrupt;

pub mod interrupts;
pub mod peripherals;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn rmain() {
    pinmux::configure();
    interrupts::initialize();
    gpio::initialize();

    for i in 21..=24 {
        gpio::pin_mode((i, GpioBank::Gpio1), GpioMode::Output);
    }

    gpio::pin_mode(GPIO1_28, GpioMode::Input);

    gpio::write(GPIO1_24, true);

    loop {
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
