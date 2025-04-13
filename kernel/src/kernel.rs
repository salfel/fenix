use core::{
    alloc::{GlobalAlloc, Layout},
    convert::TryInto,
};

use crate::{
    internals::{
        sysclock::millis,
        tasks::{scheduler, TaskState},
    },
    peripherals::{
        gpio::{self},
        i2c,
    },
};
use shared::{i2c::I2cError, kernel::Syscall};
use shared::{interrupts, kernel::SyscallReturnValue};

struct SyscallError {}

#[repr(C)]
struct TrapFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
}

impl<'a> TryInto<Syscall<'a>> for &TrapFrame {
    type Error = SyscallError;

    /// Attempts to convert a trap frame into a syscall variant based on the value in register `r12`.
    ///
    /// The conversion uses the register values to decode the syscall type and its parameters:
    /// - `0`: Returns an exit syscall.
    /// - `1`: Returns a yield syscall with a saved stack pointer (`r0`), program counter (`r1`), and an optional timeout (derived from `r2`).
    /// - `2`: Returns a milliseconds syscall.
    /// - `3`: Returns a GPIO read syscall, where the pin is determined by `r1` and `r0`.
    /// - `4`: Returns a GPIO write syscall with a boolean value from `r2` and pin information from `r1` and `r0`.
    /// - `5`: Returns an I2C write syscall with the I2C device address from `r0` and a data slice constructed unsafely from `r1` and `r2`.
    /// - `6`: Returns a panic syscall.
    /// - `7`: Returns an allocation syscall with a memory layout created from `r0` (size) and `r1` (alignment).
    /// - `8`: Returns a deallocation syscall with the pointer from `r0` and a layout from `r1` and `r2`.
    ///
    /// Returns a [`Result`] containing the corresponding syscall variant on success, or a [`SyscallError`] if the syscall type is unrecognized.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a trap frame that should decode to an exit syscall.
    /// let tf = TrapFrame { r0: 0, r1: 0, r2: 0, r3: 0, r12: 0 };
    /// let syscall = tf.try_into();
    /// assert!(matches!(syscall, Ok(Syscall::Exit)));
    ///```
    fn try_into(self) -> Result<Syscall<'a>, Self::Error> {
        match self.r12 {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Yield {
                sp: self.r0,
                pc: self.r1,
                until: match self.r2 {
                    0 => None,
                    until => Some(until),
                },
            }),
            2 => Ok(Syscall::Millis),
            3 => Ok(Syscall::GpioRead {
                pin: (self.r1, self.r0.into()),
            }),
            4 => Ok(Syscall::GpioWrite {
                pin: (self.r1, self.r0.into()),
                value: self.r2 != 0,
            }),
            5 => Ok(Syscall::I2cWrite {
                address: self.r0 as u8,
                data: unsafe { core::slice::from_raw_parts(self.r1 as *mut u8, self.r2 as usize) },
            }),
            6 => Ok(Syscall::Panic),
            7 => Ok(Syscall::Alloc {
                layout: unsafe {
                    Layout::from_size_align_unchecked(self.r0 as usize, self.r1 as usize)
                },
            }),
            8 => Ok(Syscall::Dealloc {
                ptr: self.r0 as *mut u8,
                layout: unsafe {
                    Layout::from_size_align_unchecked(self.r1 as usize, self.r2 as usize)
                },
            }),
            _ => Err(SyscallError {}),
        }
    }
}

#[repr(C)]
struct SyscallReturn {
    exit: bool,
    value: SyscallReturnValue,
}

impl SyscallReturn {
    /// Returns a `SyscallReturn` configured to signal an exit condition.
    ///
    /// This function creates a new `SyscallReturn` where the `exit` flag is set to true and the
    /// return value is marked as `none`, indicating that no additional data is provided upon exit.
    ///
    /// # Examples
    ///
    /// ```
    /// let ret = SyscallReturn::exit();
    /// assert!(ret.exit);
    /// ```
    fn exit() -> Self {
        SyscallReturn {
            exit: true,
            value: SyscallReturnValue { none: () },
        }
    }

    /// Constructs a new `SyscallReturn` with the provided return value, marking it as non-terminating.
    ///
    /// This function creates a `SyscallReturn` instance with the `exit` flag set to `false` and
    /// embeds the specified `value` as its return data.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a SyscallReturn with a specific return value.
    /// let ret_val = SyscallReturnValue::millis(1234);
    /// let result = SyscallReturn::value(ret_val);
    /// assert!(!result.exit);
    /// ```
    fn value(value: SyscallReturnValue) -> Self {
        SyscallReturn { exit: false, value }
    }

    /// Returns a `SyscallReturn` representing a syscall that produces no result.
    ///
    /// This method creates a `SyscallReturn` with the `exit` flag set to `false` and the `value`
    /// field set to indicate that there is no return value.
    ///
    /// # Examples
    ///
    /// ```
    /// let ret = SyscallReturn::none();
    /// assert!(!ret.exit);
    /// // Depending on your inspection methods, further checks on `ret.value` should confirm
    /// // that it represents a `none` variant.
    /// ```
    fn none() -> Self {
        SyscallReturn {
            exit: false,
            value: SyscallReturnValue { none: () },
        }
    }
}

#[no_mangle]
/**
Handles a software interrupt by decoding and dispatching a system call based on the provided trap frame.

This function converts a trap frame—which represents the CPU state at the time of the interrupt—into a corresponding system call.
It then executes the appropriate action, such as terminating or yielding the current task, interfacing with peripherals (GPIO or I2C),
returning the system uptime in milliseconds, or managing memory allocation. An invalid trap frame that cannot be converted to a valid
system call will trigger a panic.

# Examples
```rust
// The following test simulates a Syscall::Millis call to verify that swi_handler returns a valid uptime value.
// In a real kernel, the trap frame would be populated by hardware upon a software interrupt.
#[cfg(test)]
mod tests {
    use core::convert::TryFrom;
    use super::*;

    // Dummy conversion to simulate a Syscall::Millis from a TrapFrame.
    impl TryFrom<&TrapFrame> for Syscall {
        type Error = ();
        fn try_from(_frame: &TrapFrame) -> Result<Self, Self::Error> {
            Ok(Syscall::Millis)
        }
    }

    #[test]
    fn test_swi_handler_millis() {
        let frame = TrapFrame { r0: 0, r1: 0, r2: 0, r3: 0, r12: 0 };
        let ret = swi_handler(&frame);
        if let SyscallReturnValue { millis } = ret.value {
            assert!(millis > 0, "Expected positive milliseconds value");
        } else {
            panic!("Expected a millis return value");
        }
    }
}
```
*/
extern "C" fn swi_handler(frame: &TrapFrame) -> SyscallReturn {
    let syscall: Syscall = match frame.try_into() {
        Ok(syscall) => syscall,
        Err(_) => panic!("invalid syscall"),
    };

    match syscall {
        Syscall::Exit => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.terminate();
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Yield {
            sp,
            pc,
            until: None,
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Stored;
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Yield {
            sp,
            pc,
            until: Some(until),
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Waiting { until };
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Millis => SyscallReturn::value(SyscallReturnValue { millis: millis() }),
        Syscall::GpioWrite { pin, value } => {
            gpio::write(pin, value);

            SyscallReturn::none()
        }
        Syscall::GpioRead { pin } => {
            let value = gpio::read(pin);

            SyscallReturn::value(SyscallReturnValue { gpio_read: value })
        }
        Syscall::I2cWrite { address, data } => {
            let i2c = i2c::get_i2c();
            let mut error: I2cError = I2cError::Success;
            interrupts::enabled(|| {
                if let Err(err) = i2c.write(address, data) {
                    error = err
                }
            });

            SyscallReturn::value(SyscallReturnValue { i2c_write: error })
        }
        Syscall::Panic => {
            let scheduler = scheduler();

            if let Some(task) = scheduler.current() {
                task.terminate();
            }

            scheduler.cycle();

            SyscallReturn::exit()
        }
        Syscall::Alloc { layout } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                let ptr = unsafe { task.allocator.alloc(layout) };
                return SyscallReturn::value(SyscallReturnValue { alloc: ptr });
            }

            SyscallReturn::none()
        }
        Syscall::Dealloc { ptr, layout } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                unsafe { task.allocator.dealloc(ptr, layout) };
            }

            SyscallReturn::none()
        }
    }
}

#[no_mangle]
pub fn kernel_loop() {
    loop {
        let scheduler = scheduler();
        scheduler.switch();
    }
}
