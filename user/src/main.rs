#![no_main]
#![no_std]

use libfenix::gpio::{self, pins::GPIO1_21};

#[no_mangle]
fn _start() {
    loop {
        gpio::write(GPIO1_21, true);
    }
}
