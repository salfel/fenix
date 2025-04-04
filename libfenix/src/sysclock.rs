use crate::Syscall;
use core::arch::global_asm;

pub fn millis() -> u32 {
    let syscall = Syscall::Millis;
    syscall.call().unwrap()
}

global_asm!(
    "
    wait_store:
        stmfd sp!, {{r0-r12, lr}}

        mov r2, r0

        mrs r0, cpsr
        push {{r0}}

        mov r0, sp
        mov r1, lr
        mov r2, r2
        svc #0x1
"
);

pub fn wait(ms: u32) {
    let until = millis() + ms;
    unsafe {
        wait_store(until);
    }
}

extern "C" {
    fn wait_store(ms: u32);
}
