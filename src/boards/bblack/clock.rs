use crate::utils::wreg;

const CLOCK_ENABLE: u32 = 0x2;

pub enum ClockModule {
    CmPer = 0x44E0_0000,
    CmWkup = 0x44E0_0400,
}

#[allow(dead_code)]
pub enum FuncClock {
    // Per
    Gpio1 = 0xAC,
    Gpio2 = 0xB0,
    Gpio3 = 0xB4,

    // Wkup
    Gpio0 = 0x8,
}

impl FuncClock {
    fn clock_module(&self) -> ClockModule {
        match self {
            // Per
            FuncClock::Gpio1 => ClockModule::CmPer,
            FuncClock::Gpio2 => ClockModule::CmPer,
            FuncClock::Gpio3 => ClockModule::CmPer,

            // Wkup
            FuncClock::Gpio0 => ClockModule::CmWkup,
        }
    }

    pub fn enable(self) {
        wreg(self.clock_module() as u32 + self as u32, CLOCK_ENABLE);
    }
}

pub fn enable(clock: FuncClock) {
    clock.enable();
}
