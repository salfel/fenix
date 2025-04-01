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
    /// Creates a new, empty vector.
    ///
    /// Returns an empty vector with a null pointer, a length of 0, and a capacity of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// let vec: Vec<i32> = Vec::new();
    /// assert!(vec.is_empty());
    /// ```
    pub const fn new() -> Vec<T> {
        Vec {
            ptr: core::ptr::null_mut(),
            len: 0,
            cap: 0,
            _marker: PhantomData,
        }
    }

    /// Appends an element to the end of the vector, growing its capacity if needed.
    ///
    /// If the current number of elements equals the vector's capacity, the vector is resized by
    /// invoking the `grow()` method, which may panic if memory allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(42);
    /// assert_eq!(vec.get(0), Some(&42));
    /// ```
    pub fn push(&mut self, val: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), val);
        }

        self.len += 1;
    }

    /// Resizes the vector’s storage by allocating a new memory block with an increased capacity.
    ///
    /// When called, this method sets the new capacity to 1 if the current capacity is zero; otherwise,
    /// it doubles the existing capacity. It then allocates a new memory layout, copies any existing elements
    /// to the new memory block using a non-overlapping copy, and deallocates the old memory. The method panics
    /// if memory allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use alloc::vec::Vec; // Adjust the import path as necessary.
    /// let mut vec = Vec::<i32>::new();
    /// // Calling `grow` increases the capacity of the vector (from 0 to 1 or doubling the capacity).
    /// vec.grow();
    /// // Although `grow` does not modify the length, it updates the internal allocation.
    /// assert!(vec.len() == 0);
    /// ```
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

    /// Removes the element at the specified index, shifting subsequent elements to the left.
    ///
    /// If the given index is out of bounds, this method returns `None`. Otherwise, it removes the element,
    /// reduces the vector's length, and returns the removed element wrapped in `Some`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// // Remove the element at index 1 (value 20)
    /// let removed = vec.remove(1);
    /// assert_eq!(removed, Some(20));
    /// assert_eq!(vec.len(), 2);
    /// ```
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

        self.len -= 1;

        Some(old_value)
    }

    /// Removes and returns the last element of the vector.
    ///
    /// If the vector is empty, returns `None`. Otherwise, it reduces the vector's length
    /// and returns the previously last element.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Vec; // Replace `your_crate` with the actual crate name
    ///
    /// let mut vec = Vec::new();
    /// vec.push(5);
    /// vec.push(10);
    ///
    /// assert_eq!(vec.pop(), Some(10));
    /// assert_eq!(vec.pop(), Some(5));
    /// assert_eq!(vec.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        
        self.len -= 1;

        Some(unsafe { ptr::read(self.ptr.add(self.len)) })
    }

    /// Returns an immutable reference to the element at the specified index, if it exists.
    ///
    /// If the index is within bounds, returns `Some(&T)`; otherwise, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(42);
    /// assert_eq!(vec.get(0), Some(&42));
    /// assert_eq!(vec.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at the specified index.
    ///
    /// If `index` is within the bounds of the vector, this method returns
    /// a mutable reference to the element; otherwise, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// if let Some(elem) = vec.get_mut(1) {
    ///     *elem = 42;
    /// }
    ///
    /// assert_eq!(vec.get(1), Some(&42));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Returns the number of elements contained in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// assert_eq!(vec.len(), 0);
    ///
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// This method checks whether the vector is empty by comparing its length against zero.
    ///
    /// # Examples
    ///
    /// ```
    /// let vec = Vec::<i32>::new();
    /// assert!(vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over immutable references to the vector's elements.
    ///
    /// This method leverages the `IntoIterator` implementation for `&Vec<T>` to provide
    /// a sequential view of the elements in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let mut iter = vec.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

     /// Clears the vector by resetting its length to zero.
    ///
    /// After calling this method, the vector appears empty, although its allocated capacity remains unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(100);
    /// vec.push(200);
    /// assert_eq!(vec.len(), 2);
    ///
    /// vec.clear();
    /// assert_eq!(vec.len(), 0);
    /// ```
    pub fn clear(&mut self) {
         self.len = 0;
    }

    /// Returns a mutable iterator over the elements of the vector.
    ///
    /// The iterator yields mutable references to each element, allowing in-place modifications.
    /// 
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// // Increment each element by 1.
    /// for elem in vec.iter_mut() {
    ///     *elem += 1;
    /// }
    ///
    /// assert_eq!(vec.get(0), Some(&2));
    /// assert_eq!(vec.get(1), Some(&3));
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let end = if self.len == 0 {
            self.ptr
        } else {
            unsafe { self.ptr.add(self.len) }
        };
        IterMut {
            ptr: self.ptr,
            end,
            marker: PhantomData,
        }
    }
}

impl<T> Default for Vec<T> {
    /// Creates an empty vector.
    ///
    /// This method returns a new, empty instance of the vector, equivalent to calling [`Vec::new()`].
    ///
    /// # Examples
    ///
    /// ```
    /// let vec: Vec<i32> = Vec::default();
    /// assert!(vec.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vec<T> {
    /// Drops the vector by calling `drop_in_place` on each element and deallocating its memory.
    /// 
    /// This destructor method iterates over all elements in the vector, explicitly dropping each one. After all elements have been dropped, it deallocates the allocated memory if any (i.e., when capacity is greater than zero). This method is automatically invoked when the vector goes out of scope.
    /// 
    /// # Examples
    /// 
    /// ```
    /// {
    ///     let mut vec = Vec::new();
    ///     vec.push(42);
    ///     // `vec` is automatically dropped at the end of this scope, invoking this destructor.
    /// }
    /// ```
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

    /// Returns the next element in the iterator.
    ///
    /// If the iterator has not yet reached the end of the vector, this method returns a reference to the
    /// current element and advances the iterator's index. Once all elements have been returned, it yields
    /// `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create a new vector and add some elements.
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// // Obtain an iterator over the vector's elements.
    /// let mut iter = vec.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
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

pub struct IterMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    /// Returns the next mutable element in the iterator.
    ///
    /// This method advances the iterator and provides a mutable reference to the current element.
    /// If the iterator has reached the end, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// let mut iter = vec.iter_mut();
    /// assert_eq!(*iter.next().unwrap(), 10);
    /// assert_eq!(*iter.next().unwrap(), 20);
    /// assert_eq!(*iter.next().unwrap(), 30);
    /// assert!(iter.next().is_none());
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            unsafe {
                let item = &mut *self.ptr;
                self.ptr = self.ptr.add(1);
                Some(item)
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    /// Converts a vector reference into an iterator over its elements.
    ///
    /// This method is part of the implementation of the `IntoIterator` trait for
    /// references to `Vec<T>`. It returns an iterator that yields immutable references
    /// to the elements of the vector, starting from the first element (index 0).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// let mut iter = (&vec).into_iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self,
            index: 0,
        }
    }
}
