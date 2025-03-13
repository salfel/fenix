use core::convert::TryInto;

use crate::{
    internals::tasks::{scheduler, TaskState},
    peripherals::gpio::{self, pins::GPIO1_21},
};

enum Syscall {
    Exit,
    Yield { sp: u32, pc: u32 },
}

struct SyscallError {}

impl TryInto<Syscall> for TrapFrame {
    type Error = SyscallError;

    fn try_into(self) -> Result<Syscall, Self::Error> {
        match self.r0 {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Yield {
                sp: self.r1,
                pc: self.r2,
            }),
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
}

#[no_mangle]
extern "C" fn swi_handler(frame: TrapFrame) -> bool {
    let syscall: Syscall = match frame.try_into() {
        Ok(syscall) => syscall,
        Err(_) => return false,
    };

    match syscall {
        Syscall::Exit => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.state = TaskState::Terminated;
            }

            scheduler.cycle();

            true
        }
        Syscall::Yield { sp, pc } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Stored;
            }

            scheduler.cycle();

            true
        }
    }
}

#[no_mangle]
fn gpio() {
    gpio::write(GPIO1_21, true);
}

#[no_mangle]
fn kernel() {
    let scheduler = scheduler();
    scheduler.switch();
}
