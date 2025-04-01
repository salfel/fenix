use crate::peripherals::gpio::{
    self,
    pins::{GPIO1_21, GPIO1_22, GPIO1_23},
};

#[no_mangle]
/// Signals a data abort exception via a designated GPIO pin and halts execution by panicking.
///
/// Sets the GPIO pin `GPIO1_21` to `true` to indicate that a data abort has occurred, then immediately triggers
/// a panic to stop further execution.
///
/// # Examples
///
/// ```should_panic
/// // Calling this function will signal a data abort and panic.
/// data_abort_handler();
/// ```
fn data_abort_handler() {
    gpio::write(GPIO1_21, true);

    panic!();
}

#[no_mangle]
/// Signals a fetch abort exception by setting a dedicated GPIO pin and triggering a panic.
/// 
/// This function writes a `true` value to the fetch abort GPIO pin (GPIO1_23) to indicate that a
/// fetch abort event has occurred and then immediately panics, halting further execution.
/// 
/// # Examples
/// 
/// ```
/// use std::panic;
/// 
/// // Calling this function will result in a panic.
/// let result = panic::catch_unwind(|| {
///     fetch_abort_handler();
/// });
/// assert!(result.is_err());
/// ```
fn fetch_abort_handler() {
    gpio::write(GPIO1_23, true);

    panic!();
}

#[no_mangle]
/// Signals an undefined behavior event by setting GPIO1_22 to a high state and immediately triggering a panic.
/// 
/// This handler is invoked when the system encounters an undefined or unexpected condition. The write to GPIO1_22
/// serves as an external indication of the event before the system halts execution due to the panic.
///
/// # Examples
///
/// ```
/// // Calling undefined_handler to signal an undefined event and halt the program.
/// // This will cause the program to panic.
/// undefined_handler();
/// ```
fn undefined_handler() {
    gpio::write(GPIO1_22, true);
    
    panic!();
}

#[panic_handler]
/// Handles a panic by halting system execution in an infinite loop.
///
/// This function serves as the custom panic handler, ensuring that when a panic occurs,
/// the system is halted by entering an infinite loop. It never returns.
///
/// # Examples
///
/// ```rust
/// use core::panic::PanicInfo;
///
/// // Create a dummy PanicInfo for demonstration purposes.
/// let info = PanicInfo::message("Example panic");
///
/// // WARNING: Invoking this handler will result in an infinite loop.
/// // Uncomment the following line to test the handler at your own risk.
/// // panic(&info);
/// ```
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
