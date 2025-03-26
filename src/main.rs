#![no_std]
#![no_main]

use internals::{
    mmu, sysclock::{self, wait}, tasks::{self, create_task}
};
use kernel::kernel_loop;
use peripherals::gpio::{
    self,
    pins::{GPIO1_22, GPIO1_23, GPIO1_24},
};

pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn _start() {
    unsafe {
        setup_modes();
        setup_exceptions();
    }
    mmu::initialize();
    pinmux::configure();
    gpio::initialize();
    sysclock::initialize();
    tasks::init();

    gpio::write(GPIO1_24, true);

    create_task(user_loop);
    create_task(user_loop2);

    kernel_loop();
}

#[no_mangle]
fn user_loop() {
    loop {
        gpio::write(GPIO1_23, true);
        wait(1000);
        gpio::write(GPIO1_23, false);
        wait(1000);
    }
}

#[no_mangle]
fn user_loop2() {
    loop {
        gpio::write(GPIO1_22, false);
        wait(1000);
        gpio::write(GPIO1_22, true);
        wait(1000);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn setup_modes();
    fn setup_exceptions();
}
