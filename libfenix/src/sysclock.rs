use core::arch::asm;
use crate::Syscall;

pub fn millis() -> u32 {
    let syscall = Syscall::Millis;
    syscall.call().unwrap()
}

pub fn wait(ms: u32) {
    let until = millis() + ms;
    let sp: u32;
    let pc: u32;
    unsafe {
        asm!("stmfd sp!, {{r0-r12, lr}}", "mrs r0, cpsr", "push {{r0}}", "mov r0, sp", out("r0") sp, out("lr") pc);
    }

    let syscall = Syscall::Yield {
        sp,
        pc,
        until: Some(until),
    };
    syscall.call();
}

