use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Disables interrupts by setting the interrupt disable (I) bit in the CPSR and returns the original CPSR value.
///
/// This function reads the current CPSR, disables interrupts by setting the I-bit, and returns the original CPSR.
/// The saved CPSR value can later be used to restore the interrupts to their previous state.
///
/// # Examples
///
/// ```
/// let original_cpsr = disable_interrupts();
/// // Critical section: interrupts are disabled here.
/// // Optionally, restore interrupts using the saved CPSR value.
/// ```
fn disable_interrupts() -> u32 {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr | 0x80)
    };

    cpsr
}

/// Restores the Current Program Status Register (CPSR) to a saved state.
///
/// Writes the provided `cpsr` value back to the CPSR, typically restoring interrupt settings
/// that were previously disabled.
///
/// # Examples
///
/// ```
/// // Assume `saved_cpsr` was obtained by disabling interrupts earlier.
/// let saved_cpsr: u32 = 0xC0; // Example value representing a saved CPSR state.
/// restore_cpsr(saved_cpsr);
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
