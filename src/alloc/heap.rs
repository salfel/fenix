use core::{alloc::GlobalAlloc, ptr};

use crate::sync::mutex::Mutex;

extern crate alloc;

/// # Safety
///
/// This function is unsafe because it is an allocation function.
/// Allocates a block of memory from the global bump allocator using the specified layout.
///
/// # Safety
///
/// This function is unsafe because the caller must ensure that the allocated memory is used
/// in accordance with the layout's requirements and is valid for the type of data being allocated.
///
/// Returns a pointer to the beginning of the allocated memory, or a null pointer if allocation fails.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// unsafe {
///     let layout = Layout::from_size_align(16, 8).unwrap();
///     let ptr = alloc(layout);
///     assert!(!ptr.is_null());
/// }
/// ```pub unsafe fn alloc(layout: core::alloc::Layout) -> *mut u8 {
    let allocator = &raw mut ALLOCATOR;

    (*allocator).alloc(layout)
}

/// # Safety
///
/// This function is unsafe because it is an deallocation function.
/// Deallocates the memory block pointed to by `ptr` using the specified `layout`.
///
/// # Safety
///
/// This function is unsafe because an incorrect pointer or layout can lead to undefined behavior.
/// The caller must ensure that:
/// - `ptr` is a valid pointer previously returned by an allocation with the same layout,
/// - `layout` accurately describes the memory block,
/// - the memory is not accessed after deallocation.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// unsafe {
///     // Allocate memory (assuming an appropriate allocation function exists)
///     let layout = Layout::from_size_align(128, 8).unwrap();
///     let ptr = alloc(layout);
///     // Use the allocated memory...
///     dealloc(ptr, layout);
/// }
/// ```pub unsafe fn dealloc(ptr: *mut u8, layout: core::alloc::Layout) {
    let allocator = &raw mut ALLOCATOR;

    (*allocator).dealloc(ptr, layout)
}

/// Initializes the global bump allocator with the specified heap boundaries.
///
/// This function retrieves the heap's start and end addresses from the external static variables
/// `heap_start` and `heap_end`, casts them to `usize`, and initializes the global allocator accordingly.
/// It should be called during early program initialization before any memory allocations occur.
///
/// # Examples
///
/// ```
/// // Initialize the global allocator early in the program.
/// initialize();
/// // Subsequent allocations will use the configured heap boundaries.
/// ```
pub fn initialize() {
    let allocator = &raw mut ALLOCATOR;

    unsafe {
        (*allocator).init(
            &heap_start as *const usize as usize,
            &heap_end as *const usize as usize,
        );
    }
}

#[global_allocator]
static mut ALLOCATOR: BumpAllocator = BumpAllocator::new();

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: Mutex<usize>,
}

impl BumpAllocator {
    /// Creates a new `BumpAllocator` instance with default heap boundaries and allocation pointer.
    ///
    /// The returned allocator has its `heap_start` and `heap_end` set to 0, and the next allocation pointer
    /// initialized to 0. It must be properly initialized with valid heap boundaries using the `init` method
    /// before any allocations occur.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::BumpAllocator;
    ///
    /// let allocator = BumpAllocator::new();
    /// // At this point, the allocator's heap boundaries are unset (both 0).
    /// ```
    const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: Mutex::new(0),
        }
    }

    /// Initializes the bump allocator's heap boundaries.
    ///
    /// Sets the starting and ending addresses for the heap and resets the allocation pointer to the start address.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut allocator = BumpAllocator::new();
    /// // Initialize the allocator with heap starting at 0x1000 and ending at 0x2000.
    /// allocator.init(0x1000, 0x2000);
    /// ```
    fn init(&mut self, start: usize, end: usize) {
        self.heap_start = start;
        self.heap_end = end;
        *self.next.lock() = start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    /// Allocates a block of memory from the bump allocator using the specified layout.
    ///
    /// This method locks the allocator's internal pointer, aligns it according to the layout's alignment,
    /// and computes the end of the allocation by adding the layout's size. If the adjusted allocation
    /// fits within the heap boundaries, the pointer is advanced and a pointer to the allocated memory
    /// is returned. Otherwise, it returns a null pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it manipulates raw pointers without any lifetime guarantees.
    /// The caller must ensure that the provided layout accurately describes the intended allocation,
    /// and that the allocated memory is eventually deallocated appropriately.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::alloc::Layout;
    /// use my_allocator::BumpAllocator;
    ///
    /// unsafe {
    ///     let mut allocator = BumpAllocator::new();
    ///     // Initialize the allocator with a heap range, for example:
    ///     allocator.init(0x1000, 0x2000);
    ///
    ///     let layout = Layout::from_size_align(64, 8).unwrap();
    ///     let ptr = allocator.alloc(layout);
    ///     assert!(!ptr.is_null(), "Allocation should succeed within the heap bounds");
    /// }
    /// ```
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut current = self.next.lock();
        let alloc_start = align_up(*current, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());

        if alloc_end > self.heap_end {
            ptr::null_mut()
        } else {
            *current = alloc_end;
            alloc_start as *mut u8
        }
    }

    /// Deallocates a block of memory.
///
/// This function is a no-op for the bump allocator and is provided solely to
/// satisfy the `GlobalAlloc` trait. Although it accepts a pointer and a memory
/// layout, it does not reclaim any memory.
///
/// # Safety
///
/// Even though this function does nothing, the caller must ensure that the
/// provided pointer and layout are valid for the intended allocation.
///
/// # Examples
///
/// ```
/// # use core::alloc::Layout;
/// # struct BumpAllocator;
/// # impl BumpAllocator {
/// #     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) { }
/// # }
/// let allocator = BumpAllocator;
/// let layout = Layout::new::<u32>();
/// unsafe {
///     // This call performs no action.
///     allocator.dealloc(core::ptr::null_mut(), layout);
/// }
/// ```
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

/// Rounds up the given address to the next multiple of the specified alignment.
/// 
/// Given an address `addr` and an alignment `align`, this function returns the smallest number
/// that is greater than or equal to `addr` and is a multiple of `align`. It assumes that `align` is a power of two.
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(align_up(5, 4), 8);
/// assert_eq!(align_up(8, 4), 8);
/// ```
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

extern "C" {
    static heap_start: usize;
    static heap_end: usize;
}
