use core::{arch::asm, convert::TryInto};

use crate::{
    internals::{
        sysclock::SYS_CLOCK,
        tasks::{scheduler, TaskState},
    },
    peripherals::gpio::{self, GpioPin},
};

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

struct SyscallError {}

#[repr(C)]
struct TrapFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
}

impl TryInto<Syscall> for &TrapFrame {
    type Error = SyscallError;

    fn try_into(self) -> Result<Syscall, Self::Error> {
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
        Syscall::Millis => {
            let millis = SYS_CLOCK.lock();
            SyscallReturn::value(*millis)
        }
        Syscall::GpioWrite { pin, value } => {
            gpio::unsafe_write(pin, value);

            SyscallReturn::value(0)
        }
        Syscall::GpioRead { pin } => {
            let value = gpio::unsafe_read(pin);

            SyscallReturn::value(value as u32)
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
