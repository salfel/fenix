pub type GpioPin = (u32, GpioBank);

#[repr(u32)]
pub enum GpioBank {
    Gpio0 = 0x44E0_7000,
    Gpio1 = 0x4804_C000,
    Gpio2 = 0x481A_C000,
    Gpio3 = 0x481A_E000,
}

impl From<u32> for GpioBank {
    fn from(value: u32) -> Self {
        match value {
            0x44E0_7000 => GpioBank::Gpio0,
            0x4804_C000 => GpioBank::Gpio1,
            0x481A_C000 => GpioBank::Gpio2,
            0x481A_E000 => GpioBank::Gpio3,
            _ => panic!("invalid gpio bank"),
        }
    }
}


pub mod pins {
    use super::{GpioBank, GpioPin};

    pub const GPIO0_0: GpioPin = (0, GpioBank::Gpio0);
    pub const GPIO0_1: GpioPin = (1, GpioBank::Gpio0);
    pub const GPIO0_2: GpioPin = (2, GpioBank::Gpio0);
    pub const GPIO0_3: GpioPin = (3, GpioBank::Gpio0);
    pub const GPIO0_4: GpioPin = (4, GpioBank::Gpio0);
    pub const GPIO0_5: GpioPin = (5, GpioBank::Gpio0);
    pub const GPIO0_6: GpioPin = (6, GpioBank::Gpio0);
    pub const GPIO0_7: GpioPin = (7, GpioBank::Gpio0);
    pub const GPIO0_8: GpioPin = (8, GpioBank::Gpio0);
    pub const GPIO0_9: GpioPin = (9, GpioBank::Gpio0);
    pub const GPIO0_10: GpioPin = (10, GpioBank::Gpio0);
    pub const GPIO0_11: GpioPin = (11, GpioBank::Gpio0);
    pub const GPIO0_12: GpioPin = (12, GpioBank::Gpio0);
    pub const GPIO0_13: GpioPin = (13, GpioBank::Gpio0);
    pub const GPIO0_14: GpioPin = (14, GpioBank::Gpio0);
    pub const GPIO0_15: GpioPin = (15, GpioBank::Gpio0);
    pub const GPIO0_16: GpioPin = (16, GpioBank::Gpio0);
    pub const GPIO0_17: GpioPin = (17, GpioBank::Gpio0);
    pub const GPIO0_18: GpioPin = (18, GpioBank::Gpio0);
    pub const GPIO0_19: GpioPin = (19, GpioBank::Gpio0);
    pub const GPIO0_20: GpioPin = (20, GpioBank::Gpio0);
    pub const GPIO0_21: GpioPin = (21, GpioBank::Gpio0);
    pub const GPIO0_22: GpioPin = (22, GpioBank::Gpio0);
    pub const GPIO0_23: GpioPin = (23, GpioBank::Gpio0);
    pub const GPIO0_24: GpioPin = (24, GpioBank::Gpio0);
    pub const GPIO0_25: GpioPin = (25, GpioBank::Gpio0);
    pub const GPIO0_26: GpioPin = (26, GpioBank::Gpio0);
    pub const GPIO0_27: GpioPin = (27, GpioBank::Gpio0);
    pub const GPIO0_28: GpioPin = (28, GpioBank::Gpio0);
    pub const GPIO0_29: GpioPin = (29, GpioBank::Gpio0);
    pub const GPIO0_30: GpioPin = (30, GpioBank::Gpio0);
    pub const GPIO0_31: GpioPin = (31, GpioBank::Gpio0);

    pub const GPIO1_0: GpioPin = (0, GpioBank::Gpio1);
    pub const GPIO1_1: GpioPin = (1, GpioBank::Gpio1);
    pub const GPIO1_2: GpioPin = (2, GpioBank::Gpio1);
    pub const GPIO1_3: GpioPin = (3, GpioBank::Gpio1);
    pub const GPIO1_4: GpioPin = (4, GpioBank::Gpio1);
    pub const GPIO1_5: GpioPin = (5, GpioBank::Gpio1);
    pub const GPIO1_6: GpioPin = (6, GpioBank::Gpio1);
    pub const GPIO1_7: GpioPin = (7, GpioBank::Gpio1);
    pub const GPIO1_8: GpioPin = (8, GpioBank::Gpio1);
    pub const GPIO1_9: GpioPin = (9, GpioBank::Gpio1);
    pub const GPIO1_10: GpioPin = (10, GpioBank::Gpio1);
    pub const GPIO1_11: GpioPin = (11, GpioBank::Gpio1);
    pub const GPIO1_12: GpioPin = (12, GpioBank::Gpio1);
    pub const GPIO1_13: GpioPin = (13, GpioBank::Gpio1);
    pub const GPIO1_14: GpioPin = (14, GpioBank::Gpio1);
    pub const GPIO1_15: GpioPin = (15, GpioBank::Gpio1);
    pub const GPIO1_16: GpioPin = (16, GpioBank::Gpio1);
    pub const GPIO1_17: GpioPin = (17, GpioBank::Gpio1);
    pub const GPIO1_18: GpioPin = (18, GpioBank::Gpio1);
    pub const GPIO1_19: GpioPin = (19, GpioBank::Gpio1);
    pub const GPIO1_20: GpioPin = (20, GpioBank::Gpio1);
    pub const GPIO1_21: GpioPin = (21, GpioBank::Gpio1);
    pub const GPIO1_22: GpioPin = (22, GpioBank::Gpio1);
    pub const GPIO1_23: GpioPin = (23, GpioBank::Gpio1);
    pub const GPIO1_24: GpioPin = (24, GpioBank::Gpio1);
    pub const GPIO1_25: GpioPin = (25, GpioBank::Gpio1);
    pub const GPIO1_26: GpioPin = (26, GpioBank::Gpio1);
    pub const GPIO1_27: GpioPin = (27, GpioBank::Gpio1);
    pub const GPIO1_28: GpioPin = (28, GpioBank::Gpio1);
    pub const GPIO1_29: GpioPin = (29, GpioBank::Gpio1);
    pub const GPIO1_30: GpioPin = (30, GpioBank::Gpio1);
    pub const GPIO1_31: GpioPin = (31, GpioBank::Gpio1);

    pub const GPIO2_0: GpioPin = (0, GpioBank::Gpio2);
    pub const GPIO2_1: GpioPin = (1, GpioBank::Gpio2);
    pub const GPIO2_2: GpioPin = (2, GpioBank::Gpio2);
    pub const GPIO2_3: GpioPin = (3, GpioBank::Gpio2);
    pub const GPIO2_4: GpioPin = (4, GpioBank::Gpio2);
    pub const GPIO2_5: GpioPin = (5, GpioBank::Gpio2);
    pub const GPIO2_6: GpioPin = (6, GpioBank::Gpio2);
    pub const GPIO2_7: GpioPin = (7, GpioBank::Gpio2);
    pub const GPIO2_8: GpioPin = (8, GpioBank::Gpio2);
    pub const GPIO2_9: GpioPin = (9, GpioBank::Gpio2);
    pub const GPIO2_10: GpioPin = (10, GpioBank::Gpio2);
    pub const GPIO2_11: GpioPin = (11, GpioBank::Gpio2);
    pub const GPIO2_12: GpioPin = (12, GpioBank::Gpio2);
    pub const GPIO2_13: GpioPin = (13, GpioBank::Gpio2);
    pub const GPIO2_14: GpioPin = (14, GpioBank::Gpio2);
    pub const GPIO2_15: GpioPin = (15, GpioBank::Gpio2);
    pub const GPIO2_16: GpioPin = (16, GpioBank::Gpio2);
    pub const GPIO2_17: GpioPin = (17, GpioBank::Gpio2);
    pub const GPIO2_18: GpioPin = (18, GpioBank::Gpio2);
    pub const GPIO2_19: GpioPin = (19, GpioBank::Gpio2);
    pub const GPIO2_20: GpioPin = (20, GpioBank::Gpio2);
    pub const GPIO2_21: GpioPin = (21, GpioBank::Gpio2);
    pub const GPIO2_22: GpioPin = (22, GpioBank::Gpio2);
    pub const GPIO2_23: GpioPin = (23, GpioBank::Gpio2);
    pub const GPIO2_24: GpioPin = (24, GpioBank::Gpio2);
    pub const GPIO2_25: GpioPin = (25, GpioBank::Gpio2);
    pub const GPIO2_26: GpioPin = (26, GpioBank::Gpio2);
    pub const GPIO2_27: GpioPin = (27, GpioBank::Gpio2);
    pub const GPIO2_28: GpioPin = (28, GpioBank::Gpio2);
    pub const GPIO2_29: GpioPin = (29, GpioBank::Gpio2);
    pub const GPIO2_30: GpioPin = (30, GpioBank::Gpio2);
    pub const GPIO2_31: GpioPin = (31, GpioBank::Gpio2);

    pub const GPIO3_0: GpioPin = (0, GpioBank::Gpio3);
    pub const GPIO3_1: GpioPin = (1, GpioBank::Gpio3);
    pub const GPIO3_2: GpioPin = (2, GpioBank::Gpio3);
    pub const GPIO3_3: GpioPin = (3, GpioBank::Gpio3);
    pub const GPIO3_4: GpioPin = (4, GpioBank::Gpio3);
    pub const GPIO3_5: GpioPin = (5, GpioBank::Gpio3);
    pub const GPIO3_6: GpioPin = (6, GpioBank::Gpio3);
    pub const GPIO3_7: GpioPin = (7, GpioBank::Gpio3);
    pub const GPIO3_8: GpioPin = (8, GpioBank::Gpio3);
    pub const GPIO3_9: GpioPin = (9, GpioBank::Gpio3);
    pub const GPIO3_10: GpioPin = (10, GpioBank::Gpio3);
    pub const GPIO3_11: GpioPin = (11, GpioBank::Gpio3);
    pub const GPIO3_12: GpioPin = (12, GpioBank::Gpio3);
    pub const GPIO3_13: GpioPin = (13, GpioBank::Gpio3);
    pub const GPIO3_14: GpioPin = (14, GpioBank::Gpio3);
    pub const GPIO3_15: GpioPin = (15, GpioBank::Gpio3);
    pub const GPIO3_16: GpioPin = (16, GpioBank::Gpio3);
    pub const GPIO3_17: GpioPin = (17, GpioBank::Gpio3);
    pub const GPIO3_18: GpioPin = (18, GpioBank::Gpio3);
    pub const GPIO3_19: GpioPin = (19, GpioBank::Gpio3);
    pub const GPIO3_20: GpioPin = (20, GpioBank::Gpio3);
    pub const GPIO3_21: GpioPin = (21, GpioBank::Gpio3);
    pub const GPIO3_22: GpioPin = (22, GpioBank::Gpio3);
    pub const GPIO3_23: GpioPin = (23, GpioBank::Gpio3);
    pub const GPIO3_24: GpioPin = (24, GpioBank::Gpio3);
    pub const GPIO3_25: GpioPin = (25, GpioBank::Gpio3);
    pub const GPIO3_26: GpioPin = (26, GpioBank::Gpio3);
    pub const GPIO3_27: GpioPin = (27, GpioBank::Gpio3);
    pub const GPIO3_28: GpioPin = (28, GpioBank::Gpio3);
    pub const GPIO3_29: GpioPin = (29, GpioBank::Gpio3);
    pub const GPIO3_30: GpioPin = (30, GpioBank::Gpio3);
    pub const GPIO3_31: GpioPin = (31, GpioBank::Gpio3);
}
