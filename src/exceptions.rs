use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_23},
};

#[no_mangle]
fn data_abort_handler() {
    gpio::write(GPIO1_21, true);
}

#[no_mangle]
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);
}
