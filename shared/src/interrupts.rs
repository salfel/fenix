use core::arch::asm;

pub fn enable_interrupts() -> u32 {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr & !0x80)
    };

    cpsr
}

pub fn disable_interrupts() -> u32 {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr | 0x80)
    };

    cpsr
}

pub fn restore_cpsr(cpsr: u32) {
    unsafe { asm!("msr cpsr_c, {0}", in(reg) cpsr) };
}

pub fn enabled<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let cpsr = enable_interrupts();
    let result = f();
    restore_cpsr(cpsr);
    result
}

/// Executes the given closure with interrupts disabled.
///
/// This function disables interrupts before running the provided closure, then restores the previous
/// interrupt state once the closure has completed. It ensures that the critical section represented by
/// the closure executes with interrupts disabled, while preserving and restoring the system's original CPSR state.
///
/// # Examples
///
/// ```
/// let result = free(|| {
///     // critical section: operations that must execute with interrupts disabled
///     42
/// });
/// assert_eq!(result, 42);
/// ```
pub fn free<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let cpsr = disable_interrupts();
    let result = f();
    restore_cpsr(cpsr);
    result
}
