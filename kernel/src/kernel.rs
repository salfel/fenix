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
    fn exit() -> Self {
        SyscallReturn {
            exit: true,
            value: SyscallReturnValue { none: () },
        }
    }

    fn value(value: SyscallReturnValue) -> Self {
        SyscallReturn { exit: false, value }
    }

    fn none() -> Self {
        SyscallReturn {
            exit: false,
            value: SyscallReturnValue { none: () },
        }
    }
}

#[no_mangle]
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
