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
    /// Constructs a new empty vector.
    ///
    /// This function creates a `Vec<T>` with no allocated memory, a length of zero, and a capacity of zero.
    /// 
    /// # Examples
    ///
    /// ```
    /// let vec: Vec<i32> = Vec::new();
    /// assert_eq!(vec.len(), 0);
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

    /// Adds an element to the end of the vector.
    ///
    /// If the vector is at full capacity, it automatically grows to accommodate the new element.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(42);
    /// assert_eq!(vec.len(), 1);
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

    /// Doubles the vector's allocated memory to accommodate additional elements.
    /// 
    /// If the current capacity is zero, allocates space for one element; otherwise, doubles the capacity.
    /// Existing elements are copied to the new memory block, and the old memory block is deallocated.
    /// This method panics if memory allocation fails.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut vec = Vec::new();
    /// // Pushing elements to trigger a reallocation (growth).
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// // Elements remain intact after growth.
    /// assert_eq!(vec.pop(), Some(3));
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

    /// Removes and returns the element at the specified index from the vector.
    ///
    /// If the index is valid, the element is removed and subsequent elements are shifted left,
    /// reducing the vector's length by one. If the index is out of bounds, the vector remains unchanged
    /// and `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// assert_eq!(vec.remove(1), Some(20));
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.remove(5), None);
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

    /// Removes and returns the last element from the vector.
    ///
    /// If the vector is non-empty, this method decrements the vector's length and returns the removed element wrapped in `Some`.
    /// If the vector is empty, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.pop(), Some(2));
    /// assert_eq!(vec.pop(), Some(1));
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
    /// If the index is less than the current length of the vector, this method returns a reference
    /// to the element at that index wrapped in `Some`. Otherwise, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vector = Vec::new();
    /// vector.push(42);
    /// assert_eq!(vector.get(0), Some(&42));
    /// assert_eq!(vector.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the element at the specified index, or `None` if the index is out of bounds.
    ///
    /// If the index is within the current length of the vector, this method returns a mutable reference to the element;
    /// otherwise, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(42);
    ///
    /// if let Some(elem) = vec.get_mut(0) {
    ///     *elem = 100;
    /// }
    ///
    /// assert_eq!(vec[0], 100);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Returns the current number of elements in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// assert_eq!(vec.len(), 0);
    ///
    /// vec.push(42);
    /// assert_eq!(vec.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
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

    /// Returns an iterator over immutable references to the elements of the vector.
    ///
    /// This method converts a reference to the vector into an iterator that yields immutable
    /// references to its elements in order.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new vector and add some elements.
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    ///
    /// // Obtain an iterator and validate its output.
    /// let mut iter = vec.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&20));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

     /// Clears the vector by resetting its length to zero, effectively removing all elements.
    ///
    /// This operation does not free the underlying memory, allowing future insertions to reuse the allocated space.
    /// Note that any cleanup of element-specific resources must be performed before calling this method.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.clear();
    /// assert_eq!(vec.len(), 0);
    /// ```
    pub fn clear(&mut self) {
         self.len = 0;
    }

    /// Returns a mutable iterator over the elements of the vector.
    /// 
    /// The iterator yields mutable references, allowing the caller to modify each element in place.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use crate::alloc::vec::Vec;
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    /// 
    /// for elem in vec.iter_mut() {
    ///     *elem *= 2;
    /// }
    /// 
    /// assert_eq!(vec.get(0), Some(&2));
    /// assert_eq!(vec.get(1), Some(&4));
    /// assert_eq!(vec.get(2), Some(&6));
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
    /// Creates a new, empty vector.
    ///
    /// This function provides a default initialization for the vector and is equivalent
    /// to calling the [`new`](Self::new) method.
    ///
    /// # Examples
    ///
    /// ```
    /// let vec: Vec<i32> = Default::default();
    /// assert!(vec.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vec<T> {
    /// Drops all elements stored in the vector and deallocates its allocated memory.
    ///
    /// This method iterates over each element in the vector, calling `drop_in_place` to properly release
    /// any resources held by the elements. After disposing of all elements, if the vector had allocated
    /// memory (i.e. its capacity is non-zero), the memory is freed using the global allocator. This
    /// destructor is automatically invoked when the vector goes out of scope.
    ///
    /// # Safety
    ///
    /// This implementation makes use of unsafe code to manipulate raw pointers and deallocate memory;
    /// it assumes that the stored elements are correctly initialized and that the memory layout is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Vec;
    ///
    /// {
    ///     let mut vec = Vec::new();
    ///     vec.push(42);
    ///     vec.push(7);
    /// } // `vec` is dropped here, releasing its resources.
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
    /// Advances the iterator and returns the current element, or `None` if no elements remain.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a custom vector and add some elements.
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// // Iterate through the elements.
    /// let mut iter = vec.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&20));
    /// assert_eq!(iter.next(), Some(&30));
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

    /// Advances the iterator and returns a mutable reference to the next element.
    ///
    /// Returns `None` when the iterator is exhausted. If there is a next element,
    /// this method returns a mutable reference to it and moves the internal pointer forward.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming the custom Vec<T> and its iter_mut implementation are in scope.
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// let mut iter = vec.iter_mut();
    /// if let Some(elem) = iter.next() {
    ///     *elem += 10;
    /// }
    ///
    /// assert_eq!(vec.get(0), Some(&11));
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

    /// Consumes the vector and returns an iterator over its elements.
    ///
    /// This method converts the vector into its corresponding iterator type, starting at index 0.
    /// The returned iterator yields each element in order until the vector is exhausted.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    ///
    /// let mut iter = vec.into_iter();
    /// assert_eq!(iter.next(), Some(10));
    /// assert_eq!(iter.next(), Some(20));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self,
            index: 0,
        }
    }
}
