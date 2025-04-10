use core::arch::asm;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

pub fn disable_interrupts() -> u32 {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr | 0x80)
    };

    cpsr
}

pub fn enable_interrupts() {
    let cpsr: u32;
    unsafe {
        asm!("mrs {0}, cpsr", out(reg) cpsr);
        asm!("msr cpsr_c, {0}", in(reg) cpsr & !0x80)
    };
}

pub fn restore_cpsr(cpsr: u32) {
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
