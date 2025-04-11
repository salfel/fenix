use core::{alloc::GlobalAlloc, ptr};

use crate::sync::mutex::Mutex;

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
    const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: Mutex::new(0),
        }
    }

    fn init(&mut self, start: usize, end: usize) {
        self.heap_start = start;
        self.heap_end = end;
        *self.next.lock() = start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
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

    // Not used until now
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

extern "C" {
    static heap_start: usize;
    static heap_end: usize;
}
