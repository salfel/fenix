use shared::alloc::heap::BumpAllocator;

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

extern "C" {
    static heap_start: usize;
    static heap_end: usize;
}
