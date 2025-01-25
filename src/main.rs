#![no_std]
#![no_main]

use interfaces::gpio::{self, GpioMode};
use interrupts::Interrupt;

pub mod interfaces;
pub mod interrupts;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn rmain() {
    pinmux::configure();
    gpio::initialize();

    for i in 21..=24 {
        gpio::pin_mode(i, GpioMode::Output);
    }

    interrupts::initialize();

    gpio::pin_mode(28, GpioMode::Input);

    gpio::write(24, true);

    loop {
        gpio::write(22, gpio::read(28));
    }
}

#[no_mangle]
fn handle_interrupt() {
    gpio::write(23, true);
    let interrupt = Interrupt::get_current();
    interrupt.execute();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
