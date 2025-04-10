use crate::Syscall;

pub fn begin_transmission(slave_address: u32) {
    let syscall = Syscall::I2cBegin { slave_address };
    syscall.call();
}

pub fn write(data: u8) {
    let syscall = Syscall::I2cWrite {
        data: &[data],
    };
    syscall.call();
}

pub fn write_buf(data: &[u8]) {
    let syscall = Syscall::I2cWrite { data };
    syscall.call();
}

pub fn write_str(data: &str) {
    let syscall = Syscall::I2cWrite {
        data: data.as_bytes(),
    };
    syscall.call();
}

pub fn write_char(data: char) {
    let syscall = Syscall::I2cWrite {
        data: &[data as u8],
    };
    syscall.call();
}

pub fn end_transmission() {
    let syscall = Syscall::I2cEnd;
    syscall.call();
}
