#![no_main]
#![no_std]

use libfenix::{gpio::{self, pins::GPIO1_21}, i2c};

#[no_mangle]
fn _start() {
    i2c::begin(0x10);
    i2c::write_buf("Hello world!".as_bytes());
    i2c::end_transmission();

    gpio::write(GPIO1_21, true);

    loop {}
}
