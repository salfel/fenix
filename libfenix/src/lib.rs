#![no_std]

pub mod gpio;
pub mod i2c;
mod sysclock;

pub use shared::kernel;

pub use sysclock::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
