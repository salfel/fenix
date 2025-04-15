#![no_std]

pub mod boards;
pub mod internals;
pub mod peripherals;
pub(crate) mod utils;
pub mod vectors;

#[inline(always)]
pub fn init() {
    vectors::init();

    peripherals::gpio::init();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
