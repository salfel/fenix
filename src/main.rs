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
/// Initializes system hardware and user tasks, then launches the kernel loop.
///
/// This function sets up critical system components required for the embedded environment, including:
/// - Operating modes and exception handling (invoked within an unsafe block)
/// - Memory Management Unit (MMU) configuration
/// - Dynamic memory allocation via heap initialization
/// - Pin multiplexing and GPIO peripheral configuration
/// - System clock setup
/// - Task management initialization
///
/// It also sets an initial GPIO state, creates user tasks for GPIO operations, and finally hands control
/// over to the kernel loop, which manages task scheduling.
///
/// # Safety
///
/// The calls to `setup_modes` and `setup_exceptions` are executed within an unsafe block. Their correct usage
/// is critical to ensure system stability.
///
/// # Examples
///
/// ```rust,no_run
/// // `_start` is intended to be called at system initialization in an embedded environment.
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

/// Repeatedly toggles the state of GPIO pin `GPIO1_22` with a 1-second delay.
/// 
/// This function enters an infinite loop where it sets `GPIO1_22` low, waits for 1000 milliseconds, 
/// sets it high, and waits again for 1000 milliseconds. It is typically used in embedded systems 
/// to create a blinking effect on the specified pin.
/// 
/// # Examples
/// 
/// ```rust
/// // Warning: This function runs indefinitely. It is intended for embedded applications.
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
