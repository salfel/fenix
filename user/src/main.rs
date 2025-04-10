#![no_main]
#![no_std]

use libfenix::{i2c, wait};

#[no_mangle]
fn _start() {
    loop {
        i2c::begin_transmission(0x10);
        i2c::write_buf("Hello world!".as_bytes());
        i2c::end_transmission();
        wait(1000);
    }
}
