use core::arch::asm;

use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23, GPIO1_24},
};

#[no_mangle]
fn data_abort_handler() {
    let dfar: u32;
    let dfsr: u32;

    #[allow(asm_sub_register)]
    unsafe {
        asm!(
            "mov {0}, r0",
            "mov {1}, r1",
            out(reg) dfar,
            out(reg) dfsr,
        );
    }

    gpio::write(GPIO1_21, dfsr & 0x1 != 0);
    gpio::write(GPIO1_22, dfsr & 0x2 != 0);
    gpio::write(GPIO1_23, dfsr & 0x4 != 0);
    gpio::write(GPIO1_24, dfsr & 0x8 != 0);
}

#[no_mangle]
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);
}
