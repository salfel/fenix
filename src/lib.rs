#![no_std]

use core::arch::asm;

pub mod boards;
pub mod peripherals;
pub(crate) mod utils;

pub fn init() {
    setup_stack();

    peripherals::gpio::init();
}

fn setup_stack() {
    let stack = unsafe { &stack_end as *const u32 };
    unsafe {
        asm!("mov sp, {}", in(reg) stack);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    static stack_end: u32;
}
