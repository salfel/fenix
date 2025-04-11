use shared::{kernel::Syscall, gpio::GpioPin};

pub fn read(pin: GpioPin) -> bool {
    let syscall = Syscall::GpioRead { pin };
    syscall.call().unwrap() != 0
}

pub fn write(pin: GpioPin, value: bool) {
    let syscall = Syscall::GpioWrite { pin, value };
    syscall.call();
}

pub use shared::gpio::pins::*;
