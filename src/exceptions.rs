use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23},
};

#[no_mangle]
/// Signals a data abort by setting the designated GPIO pin and halting execution.
///
/// This function writes a `true` value to the `GPIO1_21` pin to indicate a data abort event and then
/// triggers a panic to immediately stop further execution.
///
/// # Examples
///
/// ```
/// #[should_panic]
/// fn test_data_abort_handler() {
///     data_abort_handler();
/// }
/// ```
fn data_abort_handler() {
    gpio::write(GPIO1_21, true);

    panic!();
}

#[no_mangle]
/// Handles a fetch abort event by signaling the appropriate GPIO pin and triggering a panic.
///
/// This function writes a logical `true` value to `GPIO1_23` to indicate that a fetch abort has occurred,
/// then invokes `panic!()` to halt execution.
///
/// # Examples
///
/// The following example demonstrates that invoking this function results in a panic:
///
/// ```rust
/// # #[should_panic]
/// fetch_abort_handler();
/// ```
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);

    panic!();
}

#[no_mangle]
/// Handles an undefined exception by signaling via GPIO before triggering a panic.
///
/// This function writes a `true` value to the designated GPIO pin (GPIO1_22) to indicate an undefined exception,
/// then immediately calls `panic!()` to halt further execution. It is intended to serve as an exception handler
/// for undefined instructions.
///
/// # Examples
///
/// The following example demonstrates that calling `undefined_handler` results in a panic.
///
/// ```rust
/// #[should_panic]
/// fn test_undefined_handler() {
///     undefined_handler();
/// }
/// ```
fn undefined_handler() {
    gpio::write(GPIO1_22, true);
    
    panic!();
}

#[panic_handler]
/// Handles a panic by entering an infinite loop.
///
/// This function serves as the system's panic handler. When a panic occurs, it is automatically
/// invoked to halt further execution by looping indefinitely. The provided `PanicInfo`
/// parameter, which may contain details about the panic, is intentionally ignored.
///
/// # Examples
///
/// The panic handler is automatically used in response to a panic, so directly calling it
/// is generally not recommended. For demonstration purposes, the example below shows that
/// invoking this function will result in an infinite loop:
///
/// ```rust,no_run
/// use core::panic::PanicInfo;
///
/// // For illustration only: constructing a valid PanicInfo is non-trivial.
/// // This unsafe dummy value is used solely to demonstrate the call.
/// let dummy_info: &PanicInfo = unsafe { &*(0 as *const PanicInfo) };
///
/// // Calling the panic handler will loop indefinitely.
/// panic(dummy_info);
/// ```
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
