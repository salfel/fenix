use shared::{gpio::GpioPin, kernel::Syscall};

/// Reads the current logic level of the specified GPIO pin by executing a `GpioRead` syscall.
/// 
/// This function creates a syscall for reading the state of the provided GPIO pin and executes it within an unsafe block.
/// The returned boolean indicates the pin's state (`true` for high, `false` for low). Note that the call will panic
/// if the underlying syscall fails.
/// 
/// # Examples
///
/// ```
/// // Replace `GpioPin::Example` with an actual GPIO pin variant available in your configuration.
/// let pin = GpioPin::Example;
/// let state = read(pin);
/// println!("GPIO pin state: {}", state);
/// ```
pub fn read(pin: GpioPin) -> bool {
    let syscall = Syscall::GpioRead { pin };
    unsafe { syscall.call().unwrap().gpio_read }
}

/// Writes a boolean value to the specified GPIO pin via a syscall.
///
/// This function creates a `GpioWrite` syscall using the provided pin and value,
/// then executes the syscall to update the pin's state. No result or error handling
/// is provided; it assumes the operation always succeeds.
///
/// # Examples
///
/// ```
/// use your_crate::gpio::{write, GpioPin}; // adjust the import path as needed
///
/// let pin = GpioPin::Pin1; // example GPIO pin
/// write(pin, true);
/// ```
pub fn write(pin: GpioPin, value: bool) {
    let syscall = Syscall::GpioWrite { pin, value };
    syscall.call();
}

pub use shared::gpio::pins::*;
