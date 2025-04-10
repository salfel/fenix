use crate::Syscall;

pub fn begin(slave_address: u32) {
    let syscall = Syscall::I2cBegin { slave_address };
    syscall.call();
}

pub fn write_buf(data: &[u8]) {
    let syscall = Syscall::I2cWrite { data };
    syscall.call();
}

pub fn end_transmission() {
    let syscall = Syscall::I2cEnd;
    syscall.call();
}
