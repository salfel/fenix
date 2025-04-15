#![no_std]

pub(crate) mod boards;
pub mod internals;
pub mod peripherals;
pub mod tasks;
pub(crate) mod utils;
pub(crate) mod vectors;

#[inline(always)]
pub fn init() {
    vectors::init();

    peripherals::gpio::init();
    internals::sysclock::init();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
