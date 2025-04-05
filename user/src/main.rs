#![no_main]
#![no_std]

use libfenix::{
    gpio::{self, pins::GPIO1_22},
    wait
};

#[no_mangle]
fn _start() {
    loop {
        gpio::write(GPIO1_22, false);
        wait(1000);
        gpio::write(GPIO1_22, true);
        wait(1000);
    }
}
