use core::arch::global_asm;

#[cfg(feature = "bblack")]
use crate::boards::bblack::internals::interrupts::{self, Register};

pub use interrupts::Interrupt;

global_asm!(
    "
    handle_interrupt:
        sub lr, lr, #4
        stmfd sp!, {{r0-r12, lr}}

        mrs r11, spsr
        push {{r11}}

        bl interrupt_handler

        dsb

        pop {{r11}}
        msr spsr, r11 

        ldmfd sp!, {{r0-r12, pc}}^
"
);

pub trait InterruptRegister {
    fn enable(&self, interrupt: Interrupt, priority: u8);

    fn register_handler(&mut self, interrupt: Interrupt, handler: fn());

    fn current(&self) -> Interrupt;

    fn handle_interrupt(&self);
}

static mut INTERRUPT_REGISTER: Register = Register::new();

#[allow(static_mut_refs)]
pub fn enable(interrupt: Interrupt, priority: u8) {
    unsafe {
        INTERRUPT_REGISTER.enable(interrupt, priority);
    }
}

#[allow(static_mut_refs)]
pub fn register_handler(interrupt: Interrupt, handler: fn()) {
    unsafe {
        INTERRUPT_REGISTER.register_handler(interrupt, handler);
    }
}

#[allow(static_mut_refs)]
pub fn current() -> Interrupt {
    unsafe { INTERRUPT_REGISTER.current() }
}

#[allow(static_mut_refs)]
#[no_mangle]
extern "C" fn interrupt_handler() {
    unsafe {
        INTERRUPT_REGISTER.handle_interrupt();
    }
}
