use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use crate::interrupts::{disable_interrupts, restore_cpsr};

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
