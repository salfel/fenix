use core::alloc::GlobalAlloc;

pub use shared::alloc::*;
use shared::kernel::Syscall;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator {};

struct Allocator {}

unsafe impl GlobalAlloc for Allocator {
    /// Allocates a memory block with the specified layout via a system call.
    ///
    /// This function creates a `Syscall::Alloc` with the provided layout, invokes the associated system
    /// call, and unwraps its result to extract the pointer from the `alloc` field. It returns a raw pointer
    /// to the beginning of the allocated memory block.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it interacts directly with low-level memory allocation via a
    /// system call. The caller must ensure that the provided layout is valid and that the allocated memory
    /// is eventually deallocated using the corresponding deallocation method.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::alloc::Layout;
    ///
    /// // Assuming a global allocator instance named ALLOCATOR has been declared.
    /// let layout = Layout::from_size_align(1024, 8).unwrap();
    /// let ptr = unsafe { ALLOCATOR.alloc(layout) };
    /// assert!(!ptr.is_null());
    /// // Remember to deallocate the memory when it is no longer needed.
    /// ```
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let syscall = Syscall::Alloc { layout };
        unsafe { syscall.call().unwrap().alloc }
    }

    /// Deallocates a block of memory by issuing a deallocation system call.
    ///
    /// This unsafe function constructs a system call to free the memory referenced by `ptr`
    /// using the provided `layout`. The caller must ensure that `ptr` was allocated by the corresponding
    /// allocation function and that `layout` accurately represents the allocated memory block.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `ptr` is valid and was allocated using this allocator,
    /// and that `layout` matches the memory block's layout. Failure to uphold these conditions may
    /// result in undefined behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::alloc::Layout;
    ///
    /// // Assume `ALLOCATOR` is the global allocator instance and `ptr` was allocated with it.
    /// let layout = Layout::from_size_align(256, 8).unwrap();
    /// let ptr: *mut u8 = /* pointer obtained from an allocation */;
    ///
    /// unsafe {
    ///     ALLOCATOR.dealloc(ptr, layout);
    /// }
    /// ```
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let syscall = Syscall::Dealloc { ptr, layout };
        syscall.call();
    }
}
