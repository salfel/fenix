use shared::kernel::Syscall;

pub fn write(address: u8,data: u8) {
    let syscall = Syscall::I2cWrite {
        address,
        data: &[data],
    };
    syscall.call();
}

pub fn write_buf(address: u8,data: &[u8]) {
    let syscall = Syscall::I2cWrite { address,data };
    syscall.call();
}

pub fn write_str(address: u8,data: &str) {
    let syscall = Syscall::I2cWrite {
        address,
        data: data.as_bytes(),
    };
    syscall.call();
}

pub fn write_char(address: u8,data: char) {
    let syscall = Syscall::I2cWrite {
        address,
        data: &[data as u8],
    };
    syscall.call();
}
