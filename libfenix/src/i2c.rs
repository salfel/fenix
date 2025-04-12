use shared::{i2c::I2cError, kernel::Syscall};

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
