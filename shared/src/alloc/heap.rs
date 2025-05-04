use core::{alloc::GlobalAlloc, ptr};

use crate::interrupts::CriticalSection;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: CriticalSection<usize>,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: CriticalSection::new(0),
        }
    }

    pub const fn start(start: usize) -> Self {
        Self {
            heap_start: start,
            heap_end: 0,
            next: CriticalSection::new(start),
        }
    }

    pub fn init(&mut self, start: usize, end: usize) {
        self.heap_start = start;
        self.heap_end = end;
        *self.next.lock() = start;
    }
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
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
