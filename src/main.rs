#![no_std]
#![no_main]

use interfaces::gpio::{self, GpioMode};

pub mod interfaces;
pub mod pinmux;
pub mod utils;

#[no_mangle]
pub fn rmain() {
    pinmux::configure();
    gpio::initialize();

    for i in 21..=24 {
        gpio::pin_mode(i, GpioMode::Output);
    }

    gpio::pin_mode(28, GpioMode::Input);

    gpio::write(24, true);
    gpio::write(23, true);

    loop {
        if gpio::read(28) {
            gpio::write(21, true);
        } else {
            gpio::write(21, false);
        }
    }
}

#[panic_handler]
#[no_mangle]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
