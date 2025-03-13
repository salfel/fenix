#![no_std]
#![no_main]

use internals::{
    sysclock::{self },
    tasks::{self},
};
use peripherals::gpio::{
    self,
};

pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn main() {
    pinmux::configure();
    gpio::initialize();
    sysclock::initialize();
    tasks::init();

    unsafe { kernel_loop() };
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn kernel_loop();
}
