use shared::sys::write_addr;

enum ClockModule {
    CmPer = 0x44E0_0000,
}

pub enum FuncClock {
    I2C2 = 0x44,
    I2C1 = 0x48,
    Timer7 = 0x7C,
    Timer2 = 0x80,
    Timer3 = 0x84,
    Timer4 = 0x88,
    Gpio1 = 0xAC,
    Gpio2 = 0xB0,
    Gpio3 = 0xB4,
    Timer5 = 0xEC,
    Timer6 = 0xF0,
}

impl FuncClock {
    fn clock_module(&self) -> ClockModule {
        match self {
            FuncClock::I2C2 => ClockModule::CmPer,
            FuncClock::I2C1 => ClockModule::CmPer,
            FuncClock::Timer7 => ClockModule::CmPer,
            FuncClock::Timer2 => ClockModule::CmPer,
            FuncClock::Timer3 => ClockModule::CmPer,
            FuncClock::Timer4 => ClockModule::CmPer,
            FuncClock::Gpio1 => ClockModule::CmPer,
            FuncClock::Gpio2 => ClockModule::CmPer,
            FuncClock::Gpio3 => ClockModule::CmPer,
            FuncClock::Timer5 => ClockModule::CmPer,
            FuncClock::Timer6 => ClockModule::CmPer,
        }
    }

    pub fn enable(self) {
        write_addr(self.clock_module() as u32 + self as u32, 0x2);
    }
}

pub fn enable(clock: FuncClock) {
    clock.enable();
}
