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
    gpio::initialize();

    for i in 21..=24 {
        gpio::pin_mode(i, GpioMode::Output);
    }

    gpio::pin_mode(28, GpioMode::Input);

    interrupts::initialize();

    gpio::write(24, true);

    gpio::enable_interrupt(28, GpioInterrupt::Rising, handle_rising);

    loop {
        gpio::write(22, gpio::read(28));
    }
}

fn handle_rising() {
    gpio::write(21, true);
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
