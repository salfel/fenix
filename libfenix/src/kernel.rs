use core::arch::asm;
use crate::gpio::GpioPin;

pub enum Syscall {
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
}

impl Syscall {
    pub fn call(self) -> Option<u32> {
        match self {
            Syscall::Exit => unsafe {
                asm!("svc 0x0");
                None
            },
            Syscall::Yield { sp, pc, until } => unsafe {
                asm!("svc 0x1", in("r0") sp, in("r1") pc, in("r2") until.unwrap_or(0));
                None
            },
            Syscall::Millis => unsafe {
                let millis: u32;

                asm!("push {{lr}}", "svc 0x2", "pop {{lr}}", out("r0") millis);
                Some(millis)
            },
            Syscall::GpioRead { pin: (pin, bank) } => {
                let value: u32;

                unsafe {
                    asm!("push {{lr}}", "svc 0x3", "pop {{lr}}", in("r0") bank as u32, in("r1") pin, lateout("r0") value);
                }

                Some(value)
            }
            Syscall::GpioWrite {
                pin: (pin, bank),
                value,
            } => unsafe {
                asm!("push {{lr}}", "svc 0x4", "pop {{lr}}", in("r0") bank as u32, in("r1") pin, in("r2") value as u32);
                None
            },
        }
    }
}
