#![no_main]
#![no_std]

use libfenix::{
    exit,
    gpio::{self, GPIO1_21},
    println,
};

#[no_mangle]
fn _start() {
    println!("Hello from user mode!");
    println!("This is me, Felix");
    gpio::write(GPIO1_21, true);

    exit();
}
