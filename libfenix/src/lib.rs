#![no_std]

pub mod gpio;
mod sysclock;
mod sys;
mod kernel;
pub mod i2c;

pub use sys::*;
pub use sysclock::*;
pub use kernel::*;


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
