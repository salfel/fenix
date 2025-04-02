use core::convert::TryInto;

use crate::internals::{
    sysclock::SYS_CLOCK,
    tasks::{scheduler, TaskState},
};

enum Syscall {
    Exit,
    Yield {
        sp: u32,
        pc: u32,
        until: Option<u32>,
    },
    Millis,
}

struct SyscallError {}

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
            _ => Err(SyscallError {}),
        }
    }
}

#[repr(C)]
struct TrapFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
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
#[inline(never)]
extern "C" fn test_something() -> SyscallReturn {
    SyscallReturn::exit()
}

#[no_mangle]
pub fn kernel_loop() {
    loop {
        let scheduler = scheduler();
        scheduler.switch();
    }
}
