#![no_std]
#![no_main]

use interfaces::gpio::{self, GpioInterrupt, GpioMode};
use interrupts::Interrupt;

pub mod interfaces;
pub mod interrupts;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn rmain() {
    pinmux::configure();
    interrupts::initialize();
    gpio::initialize();

    for i in 21..=24 {
        gpio::pin_mode(i, GpioMode::Output);
    }

    gpio::pin_mode(28, GpioMode::Input);

    gpio::write(24, true);

    loop {
        gpio::write(22, gpio::read(28));
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
