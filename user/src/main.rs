#![no_main]
#![no_std]

use libfenix::{gpio::{self, GPIO1_23}, wait};

#[no_mangle]
fn _start() {
    let mut status = true;
    loop {
        gpio::write(GPIO1_23, status);
        wait(1000);
        status = !status;
    }
}
