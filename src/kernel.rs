use core::convert::{TryFrom, TryInto};

use crate::internals::tasks::{scheduler, TaskState};

enum Syscall {
    Exit,
}

struct SyscallError {}

impl TryFrom<u32> for Syscall {
    type Error = SyscallError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Syscall::Exit),
            _ => Err(SyscallError {}),
        }
    }
}

#[no_mangle]
fn swi_handler(syscall: u32) -> bool {
    let syscall: Syscall = match syscall.try_into() {
        Ok(syscall) => syscall,
        Err(_) => return false,
    };

    match syscall {
        Syscall::Exit => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.state = TaskState::Terminated;
            }

            true
        }
    }
}

#[no_mangle]
fn kernel() {
    let scheduler = scheduler();
    scheduler.switch();
}
