#![no_std]

pub mod alloc;
pub mod gpio;
pub mod i2c;
mod sysclock;

pub use shared::kernel;
pub use sysclock::*;

use shared::kernel::Syscall;

/// Terminates the program by invoking the exit syscall.
///
/// This function creates a `Syscall::Exit` instance and calls it to immediately terminate the program.
/// Note that this function does not return.
///
/// # Examples
///
/// ```
/// // Warning: Calling `exit()` will terminate the program.
/// exit();
/// ```
pub fn exit() {
    let syscall = Syscall::Exit;
    syscall.call();
}

#[panic_handler]
/// Handles panic events by issuing a panic syscall and halting execution.
/// 
/// This function is intended to serve as the global panic handler in a no-std environment.
/// When invoked, it triggers a system call for panics and then enters an infinite loop to prevent
/// further code execution. The provided panic information is not utilized.
/// 
/// # Examples
/// 
/// Register this function as your panic handler:
/// 
/// ```
/// use core::panic::PanicInfo;
/// 
/// #[panic_handler]
/// fn panic_handler(info: &PanicInfo) -> ! {
///     // Replace `your_crate` with the actual crate name if necessary.
///     your_crate::panic(info)
/// }
/// ```
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let syscall = Syscall::Panic;
    syscall.call();

    loop {}
}

#[no_mangle]
/// Unwinds a C++ exception by triggering a panic via a system call.
/// 
/// This function serves as the unwind handler for C++ exceptions. It is marked with
/// `#[no_mangle]` to ensure it can be linked without name mangling, enabling interoperability
/// with external C++ code. When invoked, it triggers the panic syscall, terminating execution.
/// 
/// # Examples
///
/// ```should_panic
/// // Invoking this function will trigger a system panic.
/// __aeabi_unwind_cpp_pr0();
/// ```
fn __aeabi_unwind_cpp_pr0() {
    let syscall = Syscall::Panic;
    syscall.call();
}
