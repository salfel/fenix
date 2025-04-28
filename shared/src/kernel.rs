use crate::{gpio::GpioPin, i2c::I2cError};
use core::{alloc::Layout, arch::asm};

pub enum Syscall<'a> {
    Exit,
    Yield {
        sp: u32,
        pc: u32,
        until: Option<u32>,
    },
    Millis,
    GpioRead {
        pin: GpioPin,
    },
    GpioWrite {
        pin: GpioPin,
        value: bool,
    },
    I2cWrite {
        address: u8,
        data: &'a [u8],
    },
    Panic,
    Alloc {
        layout: Layout,
    },
    Dealloc {
        ptr: *mut u8,
        layout: Layout,
    },
}

impl Syscall<'_> {
    pub fn call(self) -> Option<SyscallReturnValue> {
        match self {
            Syscall::Exit => unsafe {
                asm!("svc 0x0", options(noreturn));
            },
            Syscall::Yield { sp, pc, until } => unsafe {
                asm!("svc 0x1", in("r0") sp, in("r1") pc, in("r2") until.unwrap_or(0), options(noreturn));
            },
            Syscall::Millis => unsafe {
                let millis: u32;

                asm!("svc 0x2", out("r0") millis);
                Some(SyscallReturnValue { millis })
            },
            Syscall::GpioRead { pin: (pin, bank) } => {
                let value: u32;

                unsafe {
                    asm!("svc 0x3", in("r0") bank as u32, in("r1") pin, lateout("r0") value);
                }

                Some(SyscallReturnValue {
                    gpio_read: value != 0,
                })
            }
            Syscall::GpioWrite {
                pin: (pin, bank),
                value,
            } => unsafe {
                asm!("svc 0x4", in("r0") bank as u32, in("r1") pin, in("r2") value as u32, lateout("r0") _);
                None
            },
            Syscall::I2cWrite { address, data } => unsafe {
                let error: u32;

                asm!("svc 0x5", in("r0") address, in("r1") data.as_ptr(), in("r2") data.len(), lateout("r0") error);

                Some(SyscallReturnValue {
                    i2c_write: error.into(),
                })
            },
            Syscall::Panic => unsafe {
                asm!("svc 0x6", options(noreturn));
            },
            Syscall::Alloc { layout } => unsafe {
                let ptr: u32;

                asm!("svc 0x7", in("r0") layout.size(), in("r1") layout.align(), lateout("r0") ptr);

                Some(SyscallReturnValue {
                    alloc: ptr as *mut u8,
                })
            },
            Syscall::Dealloc { ptr, layout } => unsafe {
                asm!("svc 0x8", in("r0") ptr, in("r1") layout.size(), in("r2") layout.align(), lateout("r0") _);
                None
            },
        }
    }
}

#[repr(C)]
pub union SyscallReturnValue {
    pub millis: u32,
    pub gpio_read: bool,
    pub i2c_write: I2cError,
    pub alloc: *mut u8,
    pub none: (),
}
