use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::{self},
};

use super::heap::{alloc, dealloc};

pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> Vec<T> {
    pub const fn new() -> Vec<T> {
        Vec {
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), val);
        }

        self.len += 1;
    }

    fn grow(&mut self) {
        let new_capacity = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let new_ptr = unsafe { alloc(new_layout) } as *mut T;
        if new_ptr.is_null() {
            panic!();
        }

        if self.len > 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
                let old_layout = Layout::array::<T>(self.cap).unwrap();
                dealloc(self.ptr as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.cap = new_capacity;
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        let old_value;
        unsafe {
            old_value = ptr::read(self.ptr.add(index));
            ptr::copy_nonoverlapping(
                self.ptr.add(index + 1),
                self.ptr.add(index),
                self.len - index - 1,
            );
        }

        Some(old_value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.remove(self.len - 1)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                ptr::drop_in_place(self.ptr.add(i));
            }
        }

        if self.cap > 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

unsafe impl<T: Sized + Sync> Sync for Vec<T> {}

pub struct Iter<'a, T> {
    vec: &'a Vec<T>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len {
            None
        } else {
            let result = self.vec.get(self.index);
            self.index += 1;
            result
        }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self,
            index: 0,
        }
    }
}
