use crate::sys::{read_addr, write_addr};

const IDLEST_BITS: u32 = 0x3 << 16;

#[repr(u32)]
enum ClockModule {
    CmPer = 0x44E0_0000,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum FunctionalClock {
    Gpio1 = 0xAC,
}

impl FunctionalClock {
    const fn clock_module(&self) -> ClockModule {
        match self {
            FunctionalClock::Gpio1 => ClockModule::CmPer,
        }
    }

    pub fn enable(&self) {
        write_addr(self.clock_module() as u32 + *self as u32, 0x2);

        while read_addr(self.clock_module() as u32 + *self as u32) & IDLEST_BITS != 0 {}
    }
}

pub fn enable(clock: FunctionalClock) {
    clock.enable();
}
