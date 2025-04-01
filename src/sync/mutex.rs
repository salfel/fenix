use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Disables interrupts and returns the previous CPSR value.
///
/// This function captures the current state of the system’s CPSR register and then disables interrupts
/// by setting the interrupt disable flag. The returned CPSR value can later be used with `restore_cpsr`
/// to reinstate the original interrupt configuration.
///
/// # Examples
///
/// ```
/// let previous_cpsr = disable_interrupts();
/// // Critical section code here...
/// restore_cpsr(previous_cpsr); // Restore previous interrupt state
/// ```
fn disable_interrupts() -> u32 {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr | 0x80)
    };

    cpsr
}

/// Restores the Current Program Status Register (CPSR) to a previously saved state.
///
/// This function writes the provided CPSR value into the CPSR register using inline assembly. It is
/// typically used to restore the processor's state—such as re-enabling interrupts—that was saved prior
/// to entering a critical section.
///
/// # Examples
///
/// ```
/// // Assume `previous_cpsr` was obtained from a corresponding function that disables interrupts.
/// let previous_cpsr: u32 = 0xDF; // Example CPSR value
/// restore_cpsr(previous_cpsr);
/// ```
fn restore_cpsr(cpsr: u32) {
    unsafe { asm!("msr cpsr_c, {0}", in(reg) cpsr) };
}

pub struct Mutex<T: Sized> {
    inner: UnsafeCell<T>,
}

unsafe impl<T: Sized + Sync> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        let cpsr = disable_interrupts();
        MutexGuard { mutex: self, cpsr }
    }
}

pub struct MutexGuard<'a, T: Sized> {
    mutex: &'a Mutex<T>,
    cpsr: u32,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        restore_cpsr(self.cpsr);
    }
}
