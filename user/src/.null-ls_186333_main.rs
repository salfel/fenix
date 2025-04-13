#![no_main]
#![no_std]

use libfenix::{
    exit,
    gpio::{self, GPIO1_21},
    i2c,
};

#[no_mangle]
fn _start() {
    i2c::write_str(0x10, "Hello there1");
    i2c::write_str(0x10, "Hello there2");
    gpio::write(GPIO1_21, true);

    exit();
}
