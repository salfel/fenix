use core::{arch::asm, cell::UnsafeCell, ops::{Deref, DerefMut}};

pub struct CriticalSection<T: Sized> {
    inner: UnsafeCell<T>,
}

unsafe impl<T: Sized + Sync> Sync for CriticalSection<T> {}

impl<T> CriticalSection<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> CriticalSectionGuard<T> {
        let cpsr = disable_interrupts();
        CriticalSectionGuard { mutex: self, cpsr }
    }
}

pub struct CriticalSectionGuard<'a, T: Sized> {
    mutex: &'a CriticalSection<T>,
    cpsr: u32,
}

impl<T> Deref for CriticalSectionGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<T> DerefMut for CriticalSectionGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

impl<T> Drop for CriticalSectionGuard<'_, T> {
    fn drop(&mut self) {
        restore_cpsr(self.cpsr);
    }
}

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

pub fn free<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let cpsr = disable_interrupts();
    let result = f();
    restore_cpsr(cpsr);
    result
}
