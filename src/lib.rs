#![no_std]

use tasks::create_task;

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

    let _ = create_task(idle, 255);
}

#[allow(clippy::empty_loop)]
fn idle() {
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
