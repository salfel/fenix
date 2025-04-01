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
/// Entry point for system initialization and task scheduling.
///
/// This function performs critical setup routines for the embedded system:
/// - Configures system operation modes and exception handling (within an unsafe block).
/// - Initializes essential components including the Memory Management Unit (MMU), dynamic heap, pin multiplexer, GPIO, system clock, and task management.
/// - Signals system readiness by setting a designated GPIO pin high.
/// - Creates two user-defined tasks for handling GPIO operations.
/// - Enters the kernel loop to manage ongoing task scheduling.
///
/// # Safety
///
/// The function executes hardware configuration routines within an unsafe block; ensure that the
/// underlying system hardware is properly supported and configured.
///
/// # Examples
///
/// In an embedded environment, this function is automatically invoked at startup. In a testing scenario,
/// it can be called directly to simulate system initialization:
///
/// ```rust,ignore
/// // Note: Directly invoking _start() outside its intended embedded context may lead to undefined behavior.
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

/// Continuously toggles the GPIO pin `GPIO1_22` every 1000 milliseconds.
///
/// This function alternates the state of the specified GPIO pin by first setting it low,
/// waiting for 1000 milliseconds, then setting it high and waiting another 1000 milliseconds.
/// It runs indefinitely and is designed to be run as a background task in an embedded system.
///
/// # Examples
///
/// ```no_run
/// // In an embedded application, schedule user_loop2 as a background task.
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
