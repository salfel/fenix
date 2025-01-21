#![no_std]
#![no_main]

#[no_mangle]
pub fn main() {
    let cm_per_gpio1_clkctrl = (0x44E00000 + 0xAC) as *mut u32;
    let gpio_oe = (0x4804C000 + 0x134) as *mut u32;
    let gpio_dataout = (0x4804C000 + 0x13C) as *mut u32;

    unsafe {
        *cm_per_gpio1_clkctrl = 2;
        *gpio_oe = 0;
        *gpio_dataout = 0x00F00000;
    }

    loop {}
}

#[panic_handler]
#[no_mangle]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
