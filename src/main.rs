#![no_std]
#![no_main]

use interfaces::gpio::{self, PinMode};

pub mod interfaces;
pub mod utils;

#[no_mangle]
pub fn main() {
    gpio::initialize();
    gpio::pin_mode(21, PinMode::Output);
    gpio::write(21, true);

    loop {}
}

#[panic_handler]
#[no_mangle]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
