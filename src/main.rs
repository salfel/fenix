#![no_std]
#![no_main]

use internals::{
    sysclock::{self, wait},
    tasks::{self, create_task},
};
use peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23, GPIO1_24},
};

pub mod exceptions;
pub mod internals;
pub mod interrupts;
pub mod kernel;
pub mod peripherals;
pub mod pinmux;
pub mod sys;

#[no_mangle]
pub fn main() {
    pinmux::configure();
    gpio::initialize();
    sysclock::initialize();
    tasks::init();

    gpio::write(GPIO1_24, true);

    create_task(user_loop);
    //create_task(user_loop2);

    unsafe { kernel_loop() };
}

#[no_mangle]
fn user_loop() {
    gpio::write(GPIO1_22, true);
    wait(1000);
    gpio::write(GPIO1_22, false);
}

fn user_loop2() {
    gpio::write(GPIO1_23, true);
    wait(1000);
    gpio::write(GPIO1_23, false);
}

#[no_mangle]
fn handle_interrupt() {
    let interrupt = interrupts::current();
    interrupts::execute(interrupt);
    interrupts::clear();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn kernel_loop();
}
