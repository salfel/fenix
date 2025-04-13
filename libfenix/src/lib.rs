#![no_std]

pub mod alloc;
pub mod gpio;
pub mod i2c;
mod sysclock;

pub use shared::kernel;
pub use sysclock::*;

use shared::kernel::Syscall;

pub fn exit() {
    let syscall = Syscall::Exit;
    syscall.call();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let syscall = Syscall::Panic;
    syscall.call();

    loop {}
}
