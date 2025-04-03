use crate::Syscall;

pub fn millis() -> u32 {
    let syscall = Syscall::Millis;
    syscall.call().unwrap()
}

