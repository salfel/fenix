use core::convert::{TryFrom, TryInto};

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
        Syscall::Exit => true,
    }
}

#[no_mangle]
fn kernel() {}
