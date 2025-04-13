use core::fmt::{self, Write};

use shared::{
    i2c::{I2cError, PRINT_ADDRESS}, kernel::Syscall
};

pub fn write(address: u8, data: u8) -> I2cError {
    write_buf(address, &[data])
}

pub fn write_buf(address: u8, data: &[u8]) -> I2cError {
    let syscall = Syscall::I2cWrite { address, data };
    unsafe { syscall.call().unwrap().i2c_write }
}

pub fn write_str(address: u8, data: &str) -> I2cError {
    write_buf(address, data.as_bytes())
}

pub fn write_char(address: u8, data: char) -> I2cError {
    write_buf(address, &[data as u8])
}

struct I2c {}

impl Write for I2c {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match write_str(PRINT_ADDRESS, s) {
            I2cError::Success => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}

pub fn print(args: core::fmt::Arguments<'_>) {
    let mut i2c = I2c {};
    i2c.write_fmt(args).unwrap();
}

pub fn println(args: core::fmt::Arguments<'_>) {
    let mut i2c = I2c {};
    i2c.write_fmt(format_args!("{}\n", args)).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::i2c::print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::i2c::println(format_args!($($arg)*))
    };
}
