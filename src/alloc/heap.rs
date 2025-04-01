use core::{alloc::GlobalAlloc, ptr};

use crate::sync::mutex::Mutex;

extern crate alloc;

/// # Safety
///
/// This function is unsafe because it is an allocation function.
/// Allocates a block of memory with the specified layout using the global bump allocator.
/// 
/// # Safety
/// 
/// This function is unsafe because it returns a raw pointer to uninitialized memory without enforcing type safety.
/// The caller must ensure that the allocated memory is used correctly according to the intended data type,
/// and must handle potential null pointers if the allocation fails.
/// 
/// # Arguments
/// 
/// * `layout` - A memory layout describing the size and alignment requirements for the allocation.
/// 
/// # Returns
/// 
/// A pointer to the allocated memory, or null if the allocation fails.
/// 
/// # Examples
/// 
/// ```rust
/// use core::alloc::Layout;
/// 
/// unsafe {
///     let layout = Layout::from_size_align(128, 8).unwrap();
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
/// Deallocates a memory block using the bump allocator.
///
/// This function forwards the deallocation request to the global bump allocator. In this
/// bump allocator implementation, deallocation is effectively a no-op. It is the caller's
/// responsibility to ensure that `ptr` and `layout` refer to a valid memory block previously
/// allocated by this allocator.
///
/// # Safety
///
/// The caller must ensure that:
/// - `ptr` is a valid pointer returned by a prior allocation using this allocator.
/// - `layout` accurately specifies the size and alignment of the allocated memory.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// unsafe {
///     // Allocate a block of memory.
///     let layout = Layout::from_size_align(1024, 8).unwrap();
///     let ptr = alloc(layout);
///
///     // Deallocate the block. Note: this is a no-op in the current bump allocator.
///     dealloc(ptr, layout);
/// }
/// ```pub unsafe fn dealloc(ptr: *mut u8, layout: core::alloc::Layout) {
    let allocator = &raw mut ALLOCATOR;

    (*allocator).dealloc(ptr, layout)
}

/// Initializes the global bump allocator by setting its heap boundaries.
///
/// This function configures the global allocator by invoking its `init` method with the
/// starting and ending addresses of the heap, which are obtained from the external
/// variables `heap_start` and `heap_end`. Internally, it performs the necessary pointer
/// casts in an unsafe block.
///
/// # Examples
///
/// ```
/// // Initialize the global allocator before performing any allocations.
/// initialize();
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
    /// Creates a new `BumpAllocator` with default heap settings.
    ///
    /// This constant function returns a `BumpAllocator` with `heap_start` and `heap_end` set to zero, and the allocation pointer
    /// (`next`) initialized to zero. The allocator must be explicitly initialized via the `init` method before it can be used for memory allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::BumpAllocator; // Replace `your_crate` with the actual crate name
    ///
    /// const ALLOCATOR: BumpAllocator = BumpAllocator::new();
    /// ```
    const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: Mutex::new(0),
        }
    }

    /// Initializes the allocator with the specified heap boundaries and resets the allocation pointer.
    ///
    /// This method sets the starting (`heap_start`) and ending (`heap_end`) addresses of the heap,
    /// and resets the internal pointer tracking the next free memory location to the start of the heap.
    /// It should be invoked before any memory allocations are performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::BumpAllocator;
    ///
    /// let mut allocator = BumpAllocator::new();
    /// // Initialize the allocator with a heap ranging from 0x1000 to 0x2000.
    /// allocator.init(0x1000, 0x2000);
    ///
    /// assert_eq!(allocator.heap_start, 0x1000);
    /// assert_eq!(allocator.heap_end, 0x2000);
    /// // Confirm that the next allocation pointer is correctly reset to the start of the heap.
    /// assert_eq!(*allocator.next.lock().unwrap(), 0x1000);
    /// ```
    fn init(&mut self, start: usize, end: usize) {
        self.heap_start = start;
        self.heap_end = end;
        *self.next.lock() = start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    /// Allocates a memory block using a bump allocation strategy.
    ///
    /// This method computes the next available address that satisfies the specified layout’s
    /// alignment, updates the internal bump pointer, and returns a pointer to the allocated block.
    /// If the allocation would exceed the heap boundary, a null pointer is returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs unchecked pointer arithmetic and does not
    /// guarantee that the requested memory is valid for use beyond the bump allocator's bounds.
    /// The caller must ensure that the allocator has been properly initialized and that the usage
    /// of the returned pointer does not lead to undefined behavior.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::alloc::Layout;
    /// use std::ptr;
    /// use std::sync::Mutex;
    ///
    /// // Dummy align_up function for demonstration purposes.
    /// fn align_up(addr: usize, align: usize) -> usize {
    ///     (addr + align - 1) & !(align - 1)
    /// }
    ///
    /// // A simple bump allocator for demonstration.
    /// struct BumpAllocator {
    ///     next: Mutex<usize>,
    ///     heap_end: usize,
    /// }
    ///
    /// impl BumpAllocator {
    ///     /// Unsafely allocates a memory block with the specified layout.
    ///     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    ///         let mut current = self.next.lock().unwrap();
    ///         let alloc_start = align_up(*current, layout.align());
    ///         let alloc_end = alloc_start.saturating_add(layout.size());
    ///
    ///         if alloc_end > self.heap_end {
    ///             ptr::null_mut()
    ///         } else {
    ///             *current = alloc_end;
    ///             alloc_start as *mut u8
    ///         }
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let allocator = BumpAllocator {
    ///         next: Mutex::new(0),
    ///         heap_end: 1024,
    ///     };
    ///
    ///     let layout = Layout::from_size_align(64, 8).unwrap();
    ///     let mem = unsafe { allocator.alloc(layout) };
    ///     assert!(!mem.is_null());
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

    /// Deallocates memory for the specified pointer and layout.
///
/// This no-op implementation exists because the bump allocator does not support
/// freeing individual allocations. Both the pointer and layout parameters are ignored.
///
/// # Safety
///
/// Although this function does nothing, it is marked as unsafe to satisfy the `GlobalAlloc`
/// trait requirements. Misuse in a real allocator could lead to undefined behavior.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// // In this bump allocator, deallocation is a no-op.
/// let layout = Layout::from_size_align(1024, 8).unwrap();
/// unsafe {
///     // `allocator` is assumed to be an instance of BumpAllocator.
///     allocator.dealloc(core::ptr::null_mut(), layout);
/// }
/// ```
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

/// Aligns the given address upward to the nearest multiple of the specified alignment.
///
/// Given an address `addr` and an alignment `align`, this function returns the smallest address
/// that is greater than or equal to `addr` and that is a multiple of `align`. It is commonly used
/// in memory allocation to ensure that addresses meet specific alignment requirements.
///
/// **Note:** The `align` parameter is expected to be a power of two.
///
/// # Examples
///
/// ```
/// let address = 5;
/// let alignment = 4;
/// let aligned_address = align_up(address, alignment);
/// // 5 aligned up to the nearest multiple of 4 is 8.
/// assert_eq!(aligned_address, 8);
/// ```
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

extern "C" {
    static heap_start: usize;
    static heap_end: usize;
}
