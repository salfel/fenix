#![no_std]

pub mod gpio;
pub mod i2c;
mod sysclock;

pub use shared::kernel;
pub use sysclock::*;

use shared::kernel::Syscall;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let syscall = Syscall::Panic;
    syscall.call();

    loop {}
}
