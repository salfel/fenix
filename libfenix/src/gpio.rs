use shared::{kernel::Syscall, gpio::GpioPin};

pub fn read(pin: GpioPin) -> bool {
    let syscall = Syscall::GpioRead { pin };
    unsafe {
        syscall.call().unwrap().gpio_read
    }
}

pub fn write(pin: GpioPin, value: bool) {
    let syscall = Syscall::GpioWrite { pin, value };
    syscall.call();
}

pub use shared::gpio::pins::*;
