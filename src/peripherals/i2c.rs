use crate::{
    internals::clock::FuncClock,
    interrupts::{clear, Interrupt},
    sys::{clear_bit, read_addr, set_bit, write_addr, write_or},
};

use super::gpio::{
    self,
    pins::GPIO1_23,
};

const I2C_IRQSTATUS_RAW: u32 = 0x24;
const I2C_IRQSTATUS: u32 = 0x28;
const I2C_IRQENABLE_SET: u32 = 0x2C;
const I2C_IRQENABLE_CLR: u32 = 0x30;

const I2C_SYSTEM_CONFIGURATION: u32 = 0x10;
const I2C_SYSTEM_STATUS: u32 = 0x90;

const I2C_CONFIGURATION: u32 = 0xA4;
const I2C_OWN_ADDRESS: u32 = 0xA8;
const I2C_SLAVE_ADDRESS: u32 = 0xAC;
const I2C_PRESCALER: u32 = 0xB0;
const I2C_SCL_LOW_TIME: u32 = 0xB4;
const I2C_SCL_HIGH_TIME: u32 = 0xB8;

const I2C_SYSTEST: u32 = 0xBC;

const SYSCLOCK: u32 = 12_000_000;

static mut I2C_MODULES: &mut [Option<I2C>; 3] = &mut [const { None }; 3];

pub fn register_i2c_module(module: I2CModule, mode: Mode) {
    unsafe {
        I2C_MODULES[module.index()] = Some(I2C::new(module, ClockSpeed::Normal, mode));
    }
}

pub fn get_i2c_module(module: I2CModule) -> Option<I2C> {
    unsafe { I2C_MODULES[module.index()].take() }
}

pub struct I2C {
    module: I2CModule,
    mode: Mode,
}

impl I2C {
    fn new(module: I2CModule, clock_speed: ClockSpeed, mode: Mode) -> I2C {
        let i2c = I2C { module, mode };

        i2c.module.clock().enable();

        i2c.reset();
        i2c.autoidle_disable();
        i2c.configure_timings(clock_speed);
        i2c.set_own_address();
        i2c.set_mode();
        i2c.enable();

        while read_addr(i2c.module as u32 + I2C_SYSTEM_STATUS) & 0x1 == 0 {}

        i2c
    }

    fn reset(&self) {
        set_bit(self.module as u32 + I2C_SYSTEM_CONFIGURATION, 1);

        clear_bit(self.module as u32 + I2C_CONFIGURATION, 15);
    }

    fn enable(&self) {
        set_bit(self.module as u32 + I2C_CONFIGURATION, 15);
    }

    fn autoidle_disable(&self) {
        clear_bit(self.module as u32 + I2C_SYSTEM_CONFIGURATION, 0);
    }

    fn set_own_address(&self) {
        write_addr(self.module as u32, 0x0);
    }

    fn configure_timings(&self, clock_speed: ClockSpeed) {
        write_addr(self.module as u32 + I2C_PRESCALER, 0x3); // SYSCLOCK/4

        write_addr(
            self.module as u32 + I2C_SCL_LOW_TIME,
            SYSCLOCK / clock_speed as u32 / 2 - 7,
        );
        write_addr(
            self.module as u32 + I2C_SCL_HIGH_TIME,
            SYSCLOCK / clock_speed as u32 / 2 - 5,
        );
    }

    fn set_mode(&self) {
        let mut value = read_addr(self.module as u32 + I2C_CONFIGURATION);
        match self.mode {
            Mode::MasterReceive => value |= 1 << 10,
            Mode::MasterTransmit => value |= 1 << 10 | 1 << 9,
            Mode::SlaveTransmit => value |= 1 << 9,
        }

        write_addr(self.module as u32 + I2C_CONFIGURATION, value);
    }

    pub fn begin(&self) {
        while read_addr(self.module as u32 + I2C_IRQSTATUS_RAW) & 1 << 12 != 0x0 {}

        write_or(self.module as u32 + I2C_CONFIGURATION, 0x3);
    }

    pub fn start_testing(&self, test_mode: TestMode) {
        write_or(
            self.module as u32 + I2C_SYSTEST,
            1 << 15 | (test_mode as u32) << 12,
        );
    }

    pub fn check_scl(&self) {
        if read_addr(self.module as u32 + I2C_SYSTEST) & 1 << 7 == 0 {
            gpio::write(GPIO1_23, true);
        }
    }
}

impl Drop for I2C {
    fn drop(&mut self) {
        unsafe {
            I2C_MODULES[self.module.index()] = Some(core::ptr::read(self));
        }
    }
}

pub enum Mode {
    MasterReceive,
    MasterTransmit,
    SlaveTransmit,
}

enum I2CInterrupt {
    Arbitration = 0,
    NoAcknowledge = 1,
    RegistersReadyForAccess = 2,
    Receive = 3,
    Transmit = 4,
    GeneralCall = 5,
    StartCondition = 6,
    AccessError = 7,
    BusFree = 8,
    AddressedAsSlave = 9,
    ReceiveDraining = 13,
    TransmitDraining = 14,
}

// Has values for SCLL and SCLH registers
#[derive(Clone, Copy)]
enum ClockSpeed {
    Normal = 100_000,
    Fast = 400_000,
}

pub enum TestMode {
    Functional = 0x0,
    SclCounter = 0x2,
    LoopBack = 0x3,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum I2CModule {
    I2C0 = 0x44E0_B000,
    I2C1 = 0x4802_A000,
    I2C2 = 0x4819_C000,
}

impl I2CModule {
    fn clock(&self) -> FuncClock {
        match self {
            I2CModule::I2C0 => FuncClock::I2C0,
            I2CModule::I2C1 => FuncClock::I2C1,
            I2CModule::I2C2 => FuncClock::I2C2,
        }
    }

    fn interrupt(&self) -> Interrupt {
        match self {
            I2CModule::I2C0 => Interrupt::I2C0INT,
            I2CModule::I2C1 => Interrupt::I2C1INT,
            I2CModule::I2C2 => Interrupt::I2C2INT,
        }
    }

    fn index(&self) -> usize {
        match self {
            I2CModule::I2C0 => 0,
            I2CModule::I2C1 => 1,
            I2CModule::I2C2 => 2,
        }
    }
}
