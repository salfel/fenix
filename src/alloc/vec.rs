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
    /// Creates a new, empty `Vec<T>`.
    ///
    /// This `const` function constructs an empty dynamic array with no allocated memory,
    /// initializing its length and capacity to zero. It can be evaluated in constant contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::alloc::vec::Vec;
    ///
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

    /// Appends a value to the end of the vector.
    ///
    /// If the vector's current capacity is reached, it automatically grows to accommodate additional elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
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

    /// Doubles the capacity of the vector's internal buffer.
    ///
    /// When the vector has reached its current capacity, this method calculates a new capacity (setting it to 1 if the current capacity is zero, or doubling it otherwise), allocates a new memory block, copies the existing elements into the new block, and deallocates the old memory. It panics if the allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::Vec; // adjust the import as needed
    /// let mut vec = Vec::new();
    /// 
    /// // Push elements to trigger a capacity increase.
    /// vec.push(1);
    /// vec.push(2);
    /// vec.push(3);
    ///
    /// // The vector now contains three elements.
    /// assert_eq!(vec.len(), 3);
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

    /// Removes the element at the specified index from the vector, shifting all subsequent elements to the left.
    ///
    /// If the index is within bounds, the function returns the removed element wrapped in `Some`.
    /// If the index is out of bounds, it returns `None` without modifying the vector.
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
    /// // After removal, the vector contains [10, 30]
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

        Some(old_value)
    }

    /// Removes and returns the last element from the vector.
    /// 
    /// If the vector is empty, returns `None`.
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
        self.remove(self.len - 1)
    }

    /// Returns a reference to the element at the specified index, if it exists.
    /// 
    /// If the index is within the bounds of the vector, this method returns `Some(&T)`;
    /// otherwise, it returns `None`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Create a new vector and add elements.
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// 
    /// // Retrieve element at index 0.
    /// assert_eq!(vec.get(0), Some(&10));
    /// 
    /// // Retrieve element at index 1.
    /// assert_eq!(vec.get(1), Some(&20));
    /// 
    /// // Attempting to retrieve an element beyond the vector's length returns None.
    /// assert_eq!(vec.get(2), None);
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
    /// This method verifies that the given index is within the current length of the vector. If the index is valid, it returns a mutable reference to the element at that position; otherwise, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::Vec; // Adjust the import path as necessary
    ///
    /// let mut vec = Vec::new();
    /// vec.push(10);
    ///
    /// if let Some(elem) = vec.get_mut(0) {
    ///     *elem = 20;
    /// }
    ///
    /// assert_eq!(vec.get(0), Some(&20));
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }

    /// Returns the number of elements in the vector.
    ///
    /// This method returns the current count of elements stored in the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// assert_eq!(vec.len(), 0);
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the vector is empty.
    ///
    /// Returns `true` if the vector contains no elements, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// assert!(vec.is_empty());
    /// vec.push(42);
    /// assert!(!vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over the vector's elements.
    ///
    /// The iterator yields immutable references to the elements in order.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// let mut iter = vec.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&20));
    /// assert_eq!(iter.next(), Some(&30));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

     /// Clears the vector by resetting its length to zero without deallocating the underlying memory.
    ///
    /// This operation effectively removes all elements from the vector, but retains the allocated capacity for future use.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// assert_eq!(vec.len(), 2);
    /// vec.clear();
    /// assert_eq!(vec.len(), 0);
    /// ```
    pub fn clear(&mut self) {
         self.len = 0;
    }

    /// Returns a mutable iterator over the vector's elements.
    /// 
    /// The returned iterator allows modifying each element in place while
    /// borrowing the vector mutably. It iterates from the first element up
    /// to the last element currently stored in the vector.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use your_crate::vec::Vec;
    /// 
    /// let mut vec = Vec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// 
    /// // Double each element using mutable iteration.
    /// for elem in vec.iter_mut() {
    ///     *elem *= 2;
    /// }
    /// 
    /// assert_eq!(vec.get(0), Some(&20));
    /// assert_eq!(vec.get(1), Some(&40));
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
    /// Returns a new, empty vector.
    ///
    /// This default implementation creates an empty `Vec<T>`, equivalent to calling
    /// [`Vec::new()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::Vec; // Adjust the import path as needed
    ///
    /// let vec: Vec<i32> = Vec::default();
    /// assert!(vec.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vec<T> {
    /// Drops the vector by calling the destructor for each element and deallocating its memory.
    ///
    /// When the vector goes out of scope, this method is invoked automatically to clean up its contents.
    /// It iterates over all initialized elements, dropping each via `ptr::drop_in_place`. If the vector
    /// has allocated memory (i.e., its capacity is non-zero), the memory is deallocated using the layout
    /// for type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// drop(vec); // `vec` is dropped, cleaning up its elements and freeing allocated memory.
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

    /// Advances the iterator and returns the next element.
    /// 
    /// If there is a remaining element, it returns a reference to that element and moves the iterator forward;
    /// otherwise, it returns `None`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Create a vector and obtain its iterator
    /// let vec = vec![10, 20, 30];
    /// let mut iter = vec.iter();
    /// 
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

    /// Advances the iterator and returns the next mutable element.
    ///
    /// If the iterator has remaining elements, this method returns a mutable reference to the current
    /// element and moves the iterator forward. Once all elements have been traversed, it returns `None`.
    ///
    /// # Examples
    ///
    /// ```
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
    /// This method transforms the vector into an iterator that yields its elements sequentially,
    /// starting at the first element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_module::Vec; // Adjust the import according to your project structure.
    /// let mut vec = Vec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// let mut iter = vec.into_iter();
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), Some(2));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self,
            index: 0,
        }
    }
}
