use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23},
};

#[no_mangle]
/// Signals a data abort event by setting GPIO1_21 high and triggering a panic.
///
/// This function writes a `true` value to the GPIO1_21 pin to indicate a data abort condition,
/// and then immediately calls `panic!()`, causing the program to halt.
///
/// # Examples
///
/// ```should_panic
/// data_abort_handler();
/// ```
fn data_abort_handler() {
    gpio::write(GPIO1_21, true);

    panic!();
}

#[no_mangle]
/// Handles a fetch abort exception by signaling the event via GPIO and halting execution.
///
/// This function writes a `true` value to the GPIO pin `GPIO1_23` to indicate that a fetch abort has occurred,
/// then immediately calls `panic!()` to transition the system into a panic state.
///
/// # Examples
///
/// ```rust
/// #[should_panic]
/// fn test_fetch_abort_handler() {
///     fetch_abort_handler();
/// }
/// ```
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);

    panic!();
}

#[no_mangle]
/// Signals an undefined behavior condition via GPIO and panics.
///
/// This function writes `true` to the `GPIO1_22` pin to indicate an undefined exception,
/// then immediately invokes a panic, halting further execution.
///
/// # Examples
///
/// ```should_panic
/// // Calling `undefined_handler` will trigger a panic due to an undefined behavior condition.
/// undefined_handler();
/// ```
fn undefined_handler() {
    gpio::write(GPIO1_22, true);
    
    panic!();
}

#[panic_handler]
/// Custom panic handler that halts execution by entering an infinite loop.
///
/// This function is automatically invoked when a panic occurs. It does not return,
/// ensuring that the system remains halted after a panic.
///
/// # Examples
///
/// ```rust
/// // Triggering a panic will invoke this custom panic handler, resulting in an infinite loop.
/// // The test is marked with #[ignore] to prevent it from running during normal test execution.
/// #[cfg(test)]
/// #[test]
/// #[ignore]
/// #[should_panic]
/// fn test_panic_handler() {
///     // This panic triggers the custom panic handler.
///     panic!("Trigger panic to test custom handler");
/// }
/// ```
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
