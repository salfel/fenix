use crate::{
    boards::bblack::clock::{self, FuncClock},
    peripherals::gpio::{GpioMode, GpioRegister},
    utils::{rbit, wbit, wreg},
};

const GPIO_OE: u32 = 0x134;
const GPIO_DATAIN: u32 = 0x138;
const GPIO_DATAOUT: u32 = 0x13C;

pub enum GpioBank {
    Bank0 = 0x44E0_7000,
    Bank1 = 0x4804_C000,
    Bank2 = 0x481A_C000,
    Bank3 = 0x481A_E000,
}

pub(crate) struct Register;

impl Register {
    pub const fn new() -> Self {
        Register {}
    }
}

impl GpioRegister for Register {
    type Bank = GpioBank;

    fn init(&mut self) {
        clock::enable(FuncClock::Gpio1);
        clock::enable(FuncClock::Gpio2);
        clock::enable(FuncClock::Gpio3);
        clock::enable(FuncClock::Gpio0);

        wreg(GpioBank::Bank1 as u32 + GPIO_OE, !0);

        for i in 21..=24 {
            // enable gpio to power leds on the board
            self.pin_mode(i, GpioBank::Bank1, GpioMode::Output);
        }
    }

    fn pin_mode(&mut self, pin: u8, bank: Self::Bank, mode: GpioMode) {
        match mode {
            GpioMode::Input => {
                wbit(bank as u32 + GPIO_OE, pin as u32, true);
            }
            GpioMode::Output => {
                wbit(bank as u32 + GPIO_OE, pin as u32, false);
            }
        }
    }

    fn write(&mut self, pin: u8, bank: Self::Bank, value: bool) {
        if value {
            wbit(bank as u32 + GPIO_DATAOUT, pin as u32, true);
        } else {
            wbit(bank as u32 + GPIO_DATAOUT, pin as u32, false);
        }
    }

    fn read(&self, pin: u8, bank: Self::Bank) -> bool {
        rbit(bank as u32 + GPIO_DATAIN, pin as u32)
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

pub mod pins {
    use crate::peripherals::gpio::GpioPin;
    use super::GpioBank;

    pub const GPIO0_0: GpioPin = (0, GpioBank::Bank0);
    pub const GPIO0_1: GpioPin = (1, GpioBank::Bank0);
    pub const GPIO0_2: GpioPin = (2, GpioBank::Bank0);
    pub const GPIO0_3: GpioPin = (3, GpioBank::Bank0);
    pub const GPIO0_4: GpioPin = (4, GpioBank::Bank0);
    pub const GPIO0_5: GpioPin = (5, GpioBank::Bank0);
    pub const GPIO0_6: GpioPin = (6, GpioBank::Bank0);
    pub const GPIO0_7: GpioPin = (7, GpioBank::Bank0);
    pub const GPIO0_8: GpioPin = (8, GpioBank::Bank0);
    pub const GPIO0_9: GpioPin = (9, GpioBank::Bank0);
    pub const GPIO0_10: GpioPin = (10, GpioBank::Bank0);
    pub const GPIO0_11: GpioPin = (11, GpioBank::Bank0);
    pub const GPIO0_12: GpioPin = (12, GpioBank::Bank0);
    pub const GPIO0_13: GpioPin = (13, GpioBank::Bank0);
    pub const GPIO0_14: GpioPin = (14, GpioBank::Bank0);
    pub const GPIO0_15: GpioPin = (15, GpioBank::Bank0);
    pub const GPIO0_16: GpioPin = (16, GpioBank::Bank0);
    pub const GPIO0_17: GpioPin = (17, GpioBank::Bank0);
    pub const GPIO0_18: GpioPin = (18, GpioBank::Bank0);
    pub const GPIO0_19: GpioPin = (19, GpioBank::Bank0);
    pub const GPIO0_20: GpioPin = (20, GpioBank::Bank0);
    pub const GPIO0_21: GpioPin = (21, GpioBank::Bank0);
    pub const GPIO0_22: GpioPin = (22, GpioBank::Bank0);
    pub const GPIO0_23: GpioPin = (23, GpioBank::Bank0);
    pub const GPIO0_24: GpioPin = (24, GpioBank::Bank0);
    pub const GPIO0_25: GpioPin = (25, GpioBank::Bank0);
    pub const GPIO0_26: GpioPin = (26, GpioBank::Bank0);
    pub const GPIO0_27: GpioPin = (27, GpioBank::Bank0);
    pub const GPIO0_28: GpioPin = (28, GpioBank::Bank0);
    pub const GPIO0_29: GpioPin = (29, GpioBank::Bank0);
    pub const GPIO0_30: GpioPin = (30, GpioBank::Bank0);
    pub const GPIO0_31: GpioPin = (31, GpioBank::Bank0);

    pub const GPIO1_0: GpioPin = (0, GpioBank::Bank1);
    pub const GPIO1_1: GpioPin = (1, GpioBank::Bank1);
    pub const GPIO1_2: GpioPin = (2, GpioBank::Bank1);
    pub const GPIO1_3: GpioPin = (3, GpioBank::Bank1);
    pub const GPIO1_4: GpioPin = (4, GpioBank::Bank1);
    pub const GPIO1_5: GpioPin = (5, GpioBank::Bank1);
    pub const GPIO1_6: GpioPin = (6, GpioBank::Bank1);
    pub const GPIO1_7: GpioPin = (7, GpioBank::Bank1);
    pub const GPIO1_8: GpioPin = (8, GpioBank::Bank1);
    pub const GPIO1_9: GpioPin = (9, GpioBank::Bank1);
    pub const GPIO1_10: GpioPin = (10, GpioBank::Bank1);
    pub const GPIO1_11: GpioPin = (11, GpioBank::Bank1);
    pub const GPIO1_12: GpioPin = (12, GpioBank::Bank1);
    pub const GPIO1_13: GpioPin = (13, GpioBank::Bank1);
    pub const GPIO1_14: GpioPin = (14, GpioBank::Bank1);
    pub const GPIO1_15: GpioPin = (15, GpioBank::Bank1);
    pub const GPIO1_16: GpioPin = (16, GpioBank::Bank1);
    pub const GPIO1_17: GpioPin = (17, GpioBank::Bank1);
    pub const GPIO1_18: GpioPin = (18, GpioBank::Bank1);
    pub const GPIO1_19: GpioPin = (19, GpioBank::Bank1);
    pub const GPIO1_20: GpioPin = (20, GpioBank::Bank1);
    pub const GPIO1_21: GpioPin = (21, GpioBank::Bank1);
    pub const GPIO1_22: GpioPin = (22, GpioBank::Bank1);
    pub const GPIO1_23: GpioPin = (23, GpioBank::Bank1);
    pub const GPIO1_24: GpioPin = (24, GpioBank::Bank1);
    pub const GPIO1_25: GpioPin = (25, GpioBank::Bank1);
    pub const GPIO1_26: GpioPin = (26, GpioBank::Bank1);
    pub const GPIO1_27: GpioPin = (27, GpioBank::Bank1);
    pub const GPIO1_28: GpioPin = (28, GpioBank::Bank1);
    pub const GPIO1_29: GpioPin = (29, GpioBank::Bank1);
    pub const GPIO1_30: GpioPin = (30, GpioBank::Bank1);
    pub const GPIO1_31: GpioPin = (31, GpioBank::Bank1);

    pub const GPIO2_0: GpioPin = (0, GpioBank::Bank2);
    pub const GPIO2_1: GpioPin = (1, GpioBank::Bank2);
    pub const GPIO2_2: GpioPin = (2, GpioBank::Bank2);
    pub const GPIO2_3: GpioPin = (3, GpioBank::Bank2);
    pub const GPIO2_4: GpioPin = (4, GpioBank::Bank2);
    pub const GPIO2_5: GpioPin = (5, GpioBank::Bank2);
    pub const GPIO2_6: GpioPin = (6, GpioBank::Bank2);
    pub const GPIO2_7: GpioPin = (7, GpioBank::Bank2);
    pub const GPIO2_8: GpioPin = (8, GpioBank::Bank2);
    pub const GPIO2_9: GpioPin = (9, GpioBank::Bank2);
    pub const GPIO2_10: GpioPin = (10, GpioBank::Bank2);
    pub const GPIO2_11: GpioPin = (11, GpioBank::Bank2);
    pub const GPIO2_12: GpioPin = (12, GpioBank::Bank2);
    pub const GPIO2_13: GpioPin = (13, GpioBank::Bank2);
    pub const GPIO2_14: GpioPin = (14, GpioBank::Bank2);
    pub const GPIO2_15: GpioPin = (15, GpioBank::Bank2);
    pub const GPIO2_16: GpioPin = (16, GpioBank::Bank2);
    pub const GPIO2_17: GpioPin = (17, GpioBank::Bank2);
    pub const GPIO2_18: GpioPin = (18, GpioBank::Bank2);
    pub const GPIO2_19: GpioPin = (19, GpioBank::Bank2);
    pub const GPIO2_20: GpioPin = (20, GpioBank::Bank2);
    pub const GPIO2_21: GpioPin = (21, GpioBank::Bank2);
    pub const GPIO2_22: GpioPin = (22, GpioBank::Bank2);
    pub const GPIO2_23: GpioPin = (23, GpioBank::Bank2);
    pub const GPIO2_24: GpioPin = (24, GpioBank::Bank2);
    pub const GPIO2_25: GpioPin = (25, GpioBank::Bank2);
    pub const GPIO2_26: GpioPin = (26, GpioBank::Bank2);
    pub const GPIO2_27: GpioPin = (27, GpioBank::Bank2);
    pub const GPIO2_28: GpioPin = (28, GpioBank::Bank2);
    pub const GPIO2_29: GpioPin = (29, GpioBank::Bank2);
    pub const GPIO2_30: GpioPin = (30, GpioBank::Bank2);
    pub const GPIO2_31: GpioPin = (31, GpioBank::Bank2);

    pub const GPIO3_0: GpioPin = (0, GpioBank::Bank3);
    pub const GPIO3_1: GpioPin = (1, GpioBank::Bank3);
    pub const GPIO3_2: GpioPin = (2, GpioBank::Bank3);
    pub const GPIO3_3: GpioPin = (3, GpioBank::Bank3);
    pub const GPIO3_4: GpioPin = (4, GpioBank::Bank3);
    pub const GPIO3_5: GpioPin = (5, GpioBank::Bank3);
    pub const GPIO3_6: GpioPin = (6, GpioBank::Bank3);
    pub const GPIO3_7: GpioPin = (7, GpioBank::Bank3);
    pub const GPIO3_8: GpioPin = (8, GpioBank::Bank3);
    pub const GPIO3_9: GpioPin = (9, GpioBank::Bank3);
    pub const GPIO3_10: GpioPin = (10, GpioBank::Bank3);
    pub const GPIO3_11: GpioPin = (11, GpioBank::Bank3);
    pub const GPIO3_12: GpioPin = (12, GpioBank::Bank3);
    pub const GPIO3_13: GpioPin = (13, GpioBank::Bank3);
    pub const GPIO3_14: GpioPin = (14, GpioBank::Bank3);
    pub const GPIO3_15: GpioPin = (15, GpioBank::Bank3);
    pub const GPIO3_16: GpioPin = (16, GpioBank::Bank3);
    pub const GPIO3_17: GpioPin = (17, GpioBank::Bank3);
    pub const GPIO3_18: GpioPin = (18, GpioBank::Bank3);
    pub const GPIO3_19: GpioPin = (19, GpioBank::Bank3);
    pub const GPIO3_20: GpioPin = (20, GpioBank::Bank3);
    pub const GPIO3_21: GpioPin = (21, GpioBank::Bank3);
    pub const GPIO3_22: GpioPin = (22, GpioBank::Bank3);
    pub const GPIO3_23: GpioPin = (23, GpioBank::Bank3);
    pub const GPIO3_24: GpioPin = (24, GpioBank::Bank3);
    pub const GPIO3_25: GpioPin = (25, GpioBank::Bank3);
    pub const GPIO3_26: GpioPin = (26, GpioBank::Bank3);
    pub const GPIO3_27: GpioPin = (27, GpioBank::Bank3);
    pub const GPIO3_28: GpioPin = (28, GpioBank::Bank3);
    pub const GPIO3_29: GpioPin = (29, GpioBank::Bank3);
    pub const GPIO3_30: GpioPin = (30, GpioBank::Bank3);
    pub const GPIO3_31: GpioPin = (31, GpioBank::Bank3);
}
