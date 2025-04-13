use core::arch::global_asm;
use shared::kernel::Syscall;

/// Returns the current system time in milliseconds.
///
/// This function uses the `Syscall::Millis` variant to perform a system call that retrieves
/// the current time in milliseconds. It executes the call in an unsafe block and unwraps the
/// result to obtain the millisecond count. Panics if the underlying system call fails.
///
/// # Examples
///
/// ```
/// let current_time = millis();
/// println!("Current time in ms: {}", current_time);
/// ```
pub fn millis() -> u32 {
    let syscall = Syscall::Millis;
    unsafe { syscall.call().unwrap().millis }
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
