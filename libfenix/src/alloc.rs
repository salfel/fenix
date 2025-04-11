use core::alloc::GlobalAlloc;

pub use shared::alloc::*;
use shared::kernel::Syscall;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator {};

struct Allocator {}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let syscall = Syscall::Alloc { layout };
        syscall.call().unwrap() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let syscall = Syscall::Dealloc { ptr, layout };
        syscall.call();
    }
}
