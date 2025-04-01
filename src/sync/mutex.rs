use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Disables interrupts by setting the interrupt disable bit in the CPSR and returns the previous CPSR state.
/// 
/// This function reads the current CPSR, then uses inline assembly to set the interrupt disable bit (0x80), effectively
/// disabling interrupts. The original CPSR value is returned so that the previous interrupt state can be restored later,
/// typically by calling a complementary function such as `restore_cpsr`.
/// 
/// # Safety
/// 
/// Disabling interrupts can affect system responsiveness and lead to missed hardware interrupts if not carefully managed.
/// Ensure that interrupts are properly restored after completing critical sections.
/// 
/// # Examples
/// 
/// ```rust
/// let original_cpsr = disable_interrupts();
/// // Execute critical operations with interrupts disabled.
/// 
/// // After critical operations, restore the original CPSR state:
/// // restore_cpsr(original_cpsr);
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
/// This function writes the provided `cpsr` value into the CPSR using inline assembly,
/// typically restoring the interrupt state as it was before being disabled (e.g., by a call
/// to disable_interrupts).
///
/// # Safety
///
/// This function uses inline assembly to modify a system register and is therefore unsafe.
/// Ensure that the `cpsr` value passed in is valid and was obtained from a trusted source.
///
/// # Examples
///
/// ```
/// // Example usage: restore a previously saved CPSR state.
/// // Typically, the `saved_cpsr` value would be obtained from a call to disable_interrupts.
/// let saved_cpsr = 0x1F; // Example value
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
