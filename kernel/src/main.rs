#![no_std]
#![no_main]

use include_programs::include_programs;
use internals::{
    mmu,
    sysclock::{self},
    tasks::{self, create_task},
};
use kernel::kernel_loop;
use peripherals::{gpio, i2c};
use shared::gpio::pins::GPIO1_24;

pub mod exceptions;
pub mod heap;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;

static PROGRAMS: &[&[u8]] = include_programs!();

#[no_mangle]
pub fn _start() {
    unsafe {
        setup_modes();
        setup_caches();
        setup_exceptions();
    }
    mmu::initialize();
    heap::initialize();
    pinmux::configure();
    gpio::initialize();
    i2c::initialize();
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
    fn setup_caches();
    fn setup_exceptions();
}
