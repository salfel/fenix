#![no_main]
#![no_std]

use libfenix::{gpio::{self, pins::GPIO1_21}, sysclock::wait};

#[no_mangle]
fn _start() {
    loop {
        gpio::write(GPIO1_21, true);
        wait(1000);
        gpio::write(GPIO1_21, false);
        wait(1000);
    }
}
