#![no_std]
#![no_main]

use alloc::heap;
use include_programs::include_programs;
use internals::{
    mmu,
    sysclock::{self},
    tasks::{self, create_task},
};
use kernel::kernel_loop;
use libfenix::gpio::{self, pins::GPIO1_24};
use peripherals::gpio::initialize_gpio;

pub mod alloc;
pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sync;
pub mod sys;

static PROGRAMS: &[&[u8]] = include_programs!();

#[no_mangle]
pub fn _start() {
    unsafe {
        setup_modes();
        setup_exceptions();
    }
    mmu::initialize();
    heap::initialize();
    pinmux::configure();
    initialize_gpio();
    sysclock::initialize();
    tasks::init();

    gpio::write(GPIO1_24, true);

    for program in PROGRAMS {
        create_task(program);
    }

    kernel_loop();
}

extern "C" {
    fn setup_modes();
    fn setup_exceptions();
}
