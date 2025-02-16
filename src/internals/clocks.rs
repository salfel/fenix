use crate::sys::{write_addr, CM_PER};

#[repr(u32)]
pub enum FuncClock {
    Timer2 = 0x80
}

pub fn enable(clock: FuncClock) {
    write_addr(CM_PER + clock as u32, 0x2);
}
