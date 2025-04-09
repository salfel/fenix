#![no_std]
#![no_main]

use alloc::heap;
use include_programs::include_programs;
use internals::{
    mmu, sysclock,
    tasks::{self, create_task},
};
use kernel::kernel_loop;
use libfenix::gpio::pins::GPIO1_24;
use peripherals::{gpio, i2c};

pub mod alloc;
pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sync;

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
    gpio::initialize();
    i2c::initialize();
    sysclock::initialize();
    tasks::init();

    let message = "Hello, world!";
    let i2c = i2c::get_i2c();
    i2c.begin(0x10);
    i2c.transmit(message.as_bytes());
    i2c.end();

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
