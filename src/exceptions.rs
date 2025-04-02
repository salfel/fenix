use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23},
};

#[no_mangle]
fn data_abort_handler() {
    gpio::write(GPIO1_21, true);

    loop {}
}

#[no_mangle]
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);

    loop {}
}

#[no_mangle]
fn undefined_handler() {
    gpio::write(GPIO1_22, true);
    
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
