#![no_std]

pub mod gpio;
pub mod i2c;
mod kernel;
mod sys;
mod sysclock;

pub use kernel::*;
pub use sys::*;
pub use sysclock::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
