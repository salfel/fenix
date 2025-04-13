use core::fmt::{self, Write};

use shared::{
    i2c::{I2cError, PRINT_ADDRESS}, kernel::Syscall
};

/// Writes a single byte to the specified I2C address.
///
/// This function wraps the provided byte in a single-element slice and delegates the write
/// operation to `write_buf`. The result is an `I2cError` indicating the status of the I2C write.
///
/// # Examples
///
/// ```
/// // Write the byte 0xA5 to the I2C device at address 0x50.
/// let result = write(0x50, 0xA5);
/// // Handle `result` as needed.
/// ```
pub fn write(address: u8, data: u8) -> I2cError {
    write_buf(address, &[data])
}

/// Writes a buffer of bytes to the specified I2C address.
///
/// This function creates a syscall request for an I2C write operation using the provided
/// `address` and the byte slice `data`. It then executes the syscall in an unsafe block,
/// unwrapping the result and returning the I2C error code.
///
/// # Panics
///
/// Panics if the syscall call fails.
///
/// # Examples
///
/// ```
/// let data = [0x01, 0x02, 0x03];
/// let error = write_buf(0x50, &data);
/// // Process the returned I2cError as needed
/// ```
pub fn write_buf(address: u8, data: &[u8]) -> I2cError {
    let syscall = Syscall::I2cWrite { address, data };
    unsafe { syscall.call().unwrap().i2c_write }
}

/// Sends a string to the specified I2C device by converting it into a byte slice.
///
/// This function converts the provided string slice to its corresponding byte representation
/// and writes the bytes to the I2C device at the given address by calling `write_buf`. The
/// operation returns an `I2cError` that indicates whether the write was successful or if an error occurred.
///
/// # Examples
///
/// ```
/// let address: u8 = 0x3C;
/// let error = write_str(address, "Hello, I2C!");
/// // Handle the error as needed, for example:
/// // assert_eq!(error, I2cError::Success);
/// ```
pub fn write_str(address: u8, data: &str) -> I2cError {
    write_buf(address, data.as_bytes())
}

/// Writes a single character to the specified I2C device.
///
/// This function converts the given character into its byte representation and writes it
/// to the I2C device at the provided address via an underlying buffer writing mechanism.
///
/// # Examples
///
/// ```
/// use crate::i2c::write_char;
///
/// // Write the character 'A' to the I2C device at address 0x3C.
/// let result = write_char(0x3C, 'A');
/// // Handle the result as needed, for example by checking for success.
/// ```
pub fn write_char(address: u8, data: char) -> I2cError {
    write_buf(address, &[data as u8])
}

struct I2c {}

impl Write for I2c {
    /// Writes a string slice to the I2C device at the predefined print address.
    ///
    /// This method converts the provided string slice into its byte representation and sends it
    /// to the I2C device using the low-level write operation. It returns `Ok(())` if the write
    /// is successful or `Err(fmt::Error)` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::fmt::Write;
    ///
    /// // Instantiate the I2C interface.
    /// let mut i2c = I2c {};
    ///
    /// // Write a string to the I2C device using the write! macro.
    /// assert!(write!(i2c, "Hello, I2C!").is_ok());
    /// ```
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match write_str(PRINT_ADDRESS, s) {
            I2cError::Success => Ok(()),
            _ => Err(fmt::Error),
        }
    }
}

/// Sends formatted output over I2C to the designated device.
///
/// This function creates an I2c instance and uses it to transmit the provided formatted
/// arguments to an I2C device. It panics if the write operation fails.
///
/// # Examples
///
/// ```
/// use core::fmt::Write;
/// print(format_args!("Status: {}", "OK"));
/// ```
pub fn print(args: core::fmt::Arguments<'_>) {
    let mut i2c = I2c {};
    i2c.write_fmt(args).unwrap();
}

/// Prints formatted output with an appended newline to the I2C device.
///
/// This function creates an `I2c` instance and writes the provided formatted arguments,
/// automatically appending a newline character. It panics if the write operation fails.
///
/// # Examples
///
/// ```
/// use core::fmt::format;
///
/// // Sends "Hello, I2C!\n" to the I2C device.
/// println(format_args!("Hello, I2C!"));
/// ```
pub fn println(args: core::fmt::Arguments<'_>) {
    let mut i2c = I2c {};
    i2c.write_fmt(format_args!("{}\n", args)).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::i2c::print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::i2c::println(format_args!($($arg)*))
    };
}
