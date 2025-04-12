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
use shared::interrupts;
use shared::kernel::Syscall;

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
    value: u32,
}

impl SyscallReturn {
    fn exit() -> Self {
        SyscallReturn {
            exit: true,
            value: 0,
        }
    }

    fn value(value: u32) -> Self {
        SyscallReturn { exit: false, value }
    }

    fn none() -> Self {
        SyscallReturn {
            exit: false,
            value: 0,
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
        Syscall::Millis => SyscallReturn::value(millis()),
        Syscall::GpioWrite { pin, value } => {
            gpio::write(pin, value);

            SyscallReturn::none()
        }
        Syscall::GpioRead { pin } => {
            let value = gpio::read(pin);

            SyscallReturn::value(value as u32)
        }
        Syscall::I2cWrite { address, data } => {
            let i2c = i2c::get_i2c();
            interrupts::enabled(|| {
                // todo don't return result for now
                let _ = i2c.write(address, data);
            });

            SyscallReturn::none()
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
                SyscallReturn::value(ptr as u32);
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
