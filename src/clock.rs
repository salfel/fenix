use crate::sys::{read_addr, write_addr};

#[repr(u32)]
enum ClockModule {
    CmPer = 0x44E0_0000,
}

enum InterfaceClock {
    L4LS = 0x0,
}

pub enum FunctionalClock {
    Gpio1,
}

impl FunctionalClock {
    fn clock_module(&self) -> ClockModule {
        match self {
            FunctionalClock::Gpio1 => ClockModule::CmPer,
        }
    }

    fn offset(&self) -> u32 {
        match self {
            FunctionalClock::Gpio1 => 0xAC,
        }
    }

    fn clkactivity_mask(&self) -> u32 {
        match self {
            FunctionalClock::Gpio1 => 1 << 19,
        }
    }

    fn interface_clock(&self) -> InterfaceClock {
        match self {
            FunctionalClock::Gpio1 => InterfaceClock::L4LS,
        }
    }

    pub fn enable(&self) {
        write_addr(self.clock_module() as u32 + self.offset(), 0x2);

        while read_addr(self.clock_module() as u32 + self.interface_clock() as u32)
            & self.clkactivity_mask()
            == 0
        {}
    }
}

pub fn enable(clock: FunctionalClock) {
    clock.enable();
}
