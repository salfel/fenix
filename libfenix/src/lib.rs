#![no_std]

pub mod gpio;
mod sysclock;
mod sys;
mod kernel;

pub use sys::*;
pub use sysclock::*;
pub use kernel::*;


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    write_addr(0, 4);
    loop {}
}
