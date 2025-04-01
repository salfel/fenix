#![no_std]
#![no_main]

use alloc::heap;
use internals::{
    mmu,
    sysclock::{self, wait},
    tasks::{self, create_task},
};
use kernel::kernel_loop;
use peripherals::gpio::{
    self,
    pins::{GPIO1_22, GPIO1_23, GPIO1_24},
};

pub mod alloc;
pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sync;
pub mod sys;

#[no_mangle]
/// Initializes system components and starts the main kernel loop.
///
/// This function serves as the entry point for the embedded system. It sets up the operating
/// modes and exception handling (within an unsafe block), and initializes the memory management
/// unit, heap, pin multiplexing, GPIO, system clock, and task manager. It then configures a 
/// specific GPIO pin, spawns two user tasks, and finally enters the kernel loop for scheduling.
///
/// # Examples
///
/// ```ignore
/// // `_start` is typically invoked automatically during system startup.
/// _start();
/// ```
pub fn _start() {
    unsafe {
        setup_modes();
        setup_exceptions();
    }
    mmu::initialize();
    heap::initialize();
    pinmux::configure();
    gpio::initialize();
    sysclock::initialize();
    tasks::init();

    gpio::write(GPIO1_24, true);

    create_task(user_loop);
    create_task(user_loop2);

    kernel_loop();
}

fn user_loop() {
    loop {
        gpio::write(GPIO1_23, true);
        wait(1000);
        gpio::write(GPIO1_23, false);
        wait(1000);
    }
}

/// Continuously toggles the GPIO1_22 pin between low and high states every 1000 milliseconds.
///
/// This function enters an infinite loop where it first sets GPIO1_22 to low, waits for 1000 milliseconds,
/// then sets it to high and waits again. It is intended for use in embedded systems as a concurrent task
/// for signaling status or blinking an LED.
///
/// # Examples
///
/// ```no_run
/// // In an embedded system, user_loop2 is typically run as a concurrent task.
/// user_loop2();
/// ```
fn user_loop2() {
    loop {
        gpio::write(GPIO1_22, false);
        wait(1000);
        gpio::write(GPIO1_22, true);
        wait(1000);
    }
}

extern "C" {
    fn setup_modes();
    fn setup_exceptions();
}
