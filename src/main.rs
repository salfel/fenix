#![no_std]
#![no_main]

use internals::sysclock::{self, wait};
use peripherals::gpio::{
    self,
    pins::{GPIO1_22, GPIO1_24},
};

pub mod exceptions;
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

    gpio::write(GPIO1_24, true);

    let mut status = true;
    loop {
        wait(1000);
        gpio::write(GPIO1_22, status);
        status = !status;
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
