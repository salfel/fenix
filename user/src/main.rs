#![no_main]
#![no_std]

use libfenix::{
    exit,
    gpio::{self, GPIO1_21},
    println,
};

#[no_mangle]
/// Entry point for the application in a no_std environment.
///
/// This function prints greeting messages to the console, sets the designated GPIO pin (GPIO1_21) to high, and then terminates
/// execution by invoking the exit function. It acts as the starting point for program execution on embedded systems.
///
/// # Examples
///
/// ```
/// // Note: Calling _start() will terminate the program.
/// _start();
/// ```
fn _start() {
    println!("Hello from user mode!");
    println!("This is me, Felix");
    gpio::write(GPIO1_21, true);

    exit();
}
