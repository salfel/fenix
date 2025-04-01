use core::{alloc::GlobalAlloc, ptr};

use crate::sync::mutex::Mutex;

extern crate alloc;

/// # Safety
///
/// This function is unsafe because it is an allocation function.
/// Allocates memory using the global bump allocator based on the given layout.
///
/// This function returns a pointer to a block of memory that fulfills the requested layout.
/// It is unsafe because the caller must ensure that the allocated memory is used correctly for
/// the intended data type and that the layout provided is accurate. If the allocation fails,
/// a null pointer is returned.
///
/// # Safety
///
/// The caller is responsible for guaranteeing that the memory obtained from this function
/// is valid for the type of data being allocated.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// unsafe {
///     let layout = Layout::from_size_align(128, 16).unwrap();
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
/// Deallocates a memory block using the global bump allocator.
///
/// This function delegates deallocation to the bump allocator. Since the bump allocator
/// does not actually reclaim memory, this function does not free memory but satisfies the
/// allocator interface.
///
/// # Safety
///
/// The caller must ensure that `ptr` was allocated by this allocator and that `layout`
/// correctly describes the memory layout of the allocation. Misuse may lead to undefined behavior.
///
/// # Examples
///
/// ```
/// use std::alloc::Layout;
///
/// // In a real use case, `ptr` should be obtained from an allocation via the corresponding allocator.
/// let layout = Layout::from_size_align(128, 8).unwrap();
/// let ptr = 0x1000 as *mut u8;
///
/// unsafe {
///     dealloc(ptr, layout);
/// }
/// ```pub unsafe fn dealloc(ptr: *mut u8, layout: core::alloc::Layout) {
    let allocator = &raw mut ALLOCATOR;

    (*allocator).dealloc(ptr, layout)
}

/// Initializes the global bump allocator with the heap boundaries.
///
/// Sets up the global allocator by configuring its heap range using the external symbols
/// `heap_start` and `heap_end`. This function must be called before any heap allocations are made.
///
/// # Examples
///
/// ```
/// // Initialize the global bump allocator.
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
    /// Creates a new `BumpAllocator` instance with default zeroed heap boundaries and allocation pointer.
    /// 
    /// This constructor initializes `heap_start` and `heap_end` to zero, and sets the internal pointer (`next`)
    /// to zero as well. The allocator must be subsequently initialized with valid heap boundaries via the `init` or
    /// `initialize` functions before any memory allocation is attempted.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Create a new instance of BumpAllocator with default settings.
    /// const ALLOCATOR: BumpAllocator = BumpAllocator::new();
    /// 
    /// // The allocator requires initialization with proper heap boundaries before use:
    /// // ALLOCATOR.init(heap_start_address, heap_end_address);
    /// ```
    const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: Mutex::new(0),
        }
    }

    /// Initializes the bump allocator with a specified heap region.
    ///
    /// Sets the allocator's start and end boundaries and resets the allocation pointer
    /// to the beginning of the heap. This prepares the allocator for subsequent memory allocations.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting address of the heap.
    /// * `end` - The ending address of the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Mutex;
    ///
    /// // A simplified representation of a bump allocator for demonstration purposes.
    /// struct BumpAllocator {
    ///     heap_start: usize,
    ///     heap_end: usize,
    ///     next: Mutex<usize>,
    /// }
    ///
    /// impl BumpAllocator {
    ///     pub const fn new() -> Self {
    ///         Self {
    ///             heap_start: 0,
    ///             heap_end: 0,
    ///             next: Mutex::new(0),
    ///         }
    ///     }
    ///
    ///     fn init(&mut self, start: usize, end: usize) {
    ///         self.heap_start = start;
    ///         self.heap_end = end;
    ///         *self.next.lock().unwrap() = start;
    ///     }
    /// }
    ///
    /// let mut allocator = BumpAllocator::new();
    /// let heap_start = 0x1000;
    /// let heap_end = 0x2000;
    /// allocator.init(heap_start, heap_end);
    /// assert_eq!(*allocator.next.lock().unwrap(), heap_start);
    /// ```
    fn init(&mut self, start: usize, end: usize) {
        self.heap_start = start;
        self.heap_end = end;
        *self.next.lock() = start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    /// Attempts to allocate a memory block with the specified layout using a bump allocation strategy.
    /// 
    /// This method computes an aligned starting address based on the layout's alignment and checks whether
    /// the memory block fits within the heap boundary. If enough space is available, it advances the allocation
    /// pointer and returns a pointer to the allocated memory. Otherwise, it returns a null pointer.
    /// 
    /// # Safety
    ///
    /// Calling this function is unsafe because the caller must ensure that the provided layout is valid and
    /// that the returned pointer is used according to the layout's specifications. Misuse may lead to undefined behavior.
    /// 
    /// # Examples
    ///
    /// ```
    /// use core::alloc::Layout;
    /// use spin::Mutex;
    ///
    /// // A minimal bump allocator example for demonstration purposes.
    /// struct BumpAllocator {
    ///     heap_start: usize,
    ///     heap_end: usize,
    ///     next: Mutex<usize>,
    /// }
    ///
    /// // A helper function to align addresses upward to the nearest multiple of `align`.
    /// fn align_up(addr: usize, align: usize) -> usize {
    ///     (addr + align - 1) & !(align - 1)
    /// }
    ///
    /// impl BumpAllocator {
    ///     /// Allocates a memory block using bump allocation.
    ///     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    ///         let mut current = self.next.lock();
    ///         let alloc_start = align_up(*current, layout.align());
    ///         let alloc_end = alloc_start.saturating_add(layout.size());
    ///
    ///         if alloc_end > self.heap_end {
    ///             core::ptr::null_mut()
    ///         } else {
    ///             *current = alloc_end;
    ///             alloc_start as *mut u8
    ///         }
    ///     }
    /// }
    ///
    /// // Initialize a dummy allocator instance with a predetermined heap range.
    /// let allocator = BumpAllocator {
    ///     heap_start: 0x1000,
    ///     heap_end: 0x2000,
    ///     next: Mutex::new(0x1000),
    /// };
    ///
    /// let layout = Layout::from_size_align(64, 8).unwrap();
    /// let ptr = unsafe { allocator.alloc(layout) };
    /// assert!(!ptr.is_null());
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

    /// No-op deallocation for the bump allocator.
///
/// This method is part of the bump allocator’s implementation of the deallocation interface and is intentionally left empty,
/// as the allocator does not reclaim individual memory blocks. Memory is recovered only by resetting or replacing the allocator.
///
/// # Safety
///
/// Although marked as unsafe, this function performs no operations. The pointer and layout provided should still correspond
/// to a memory region originally allocated by this allocator.
///
/// # Examples
///
/// ```
/// use core::alloc::Layout;
///
/// unsafe {
///     // Example using the global allocator instance.
///     let dummy_ptr = core::ptr::null_mut();
///     let layout = Layout::from_size_align(64, 8).unwrap();
///     ALLOCATOR.dealloc(dummy_ptr, layout);
/// }
/// ```
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

/// Returns the largest multiple of `align` that is less than or equal to `addr`.
///
/// This function clears the lower bits of `addr` (as specified by `align - 1`) to compute an aligned address.
/// It assumes that `align` is a power of two.
///
/// # Examples
///
/// ```
/// let addr = 7;
/// // For align = 4, the largest multiple of 4 less than or equal to 7 is 4.
/// assert_eq!(align_up(addr, 4), 4);
///
/// let addr = 8;
/// // 8 is already a multiple of 4, so it remains unchanged.
/// assert_eq!(align_up(addr, 4), 8);
/// ```
fn align_up(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

extern "C" {
    static heap_start: usize;
    static heap_end: usize;
}
