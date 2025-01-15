#![no_std]
#![no_main]

#[no_mangle]
pub fn main() {
    loop {}
}

#[panic_handler]
#[no_mangle]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
