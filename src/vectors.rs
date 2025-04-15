use core::arch::{asm, global_asm};

const SUPERVISOR_MODE: u32 = 0x53;
const INTERRUPT_MODE: u32 = 0x52;

global_asm!(
    "
    .global setup_exceptions

    .align 5
    vectors:
        b main
        b .
        b .
        b .
        b .
        b .
        b handle_interrupt
        b .

    setup_exceptions:
        ldr r0, =vectors
        mcr p15, 0, r0, c12, c0, 0
        dsb

        bx lr
    "
);

pub(crate) fn init() {
    setup_stack();

    unsafe {
        setup_exceptions();
    }
}

pub fn setup_stack() {
    unsafe {
        asm!(
            "msr cpsr_c, {irq_mode}",
            "mov sp, {irq_stack}",

            "msr cpsr_c, {svc_mode}",
            "mov sp, {svc_stack}",

            svc_mode = const SUPERVISOR_MODE,
            irq_mode = const INTERRUPT_MODE,
            svc_stack = in(reg) &stack_end as *const u32,
            irq_stack = in(reg) &irq_stack_end as *const u32,
        )
    };
}

extern "C" {
    static stack_end: u32;
    static irq_stack_end: u32;

    pub fn setup_exceptions();
}
