use crate::utils::write_addr;

const CONTROL_MODULE_BASE: u32 = 0x44E10000;
pub const CONF_GPMC_BEN1: u32 = 0x878;

pub fn configure() {
    set_pin_mode(CONF_GPMC_BEN1, 7, true, PullResistor::PullDown);
}

pub fn set_pin_mode(offset: u32, mode: u32, input_enable: bool, pull_resistor: PullResistor) {
    let control_module = CONTROL_MODULE_BASE + offset;

    write_addr(
        control_module,
        mode | (pull_resistor.to_mask() << 3) | ((input_enable as u32) << 5),
    );
}

pub enum PullResistor {
    None,
    PullDown,
    PullUp,
}

impl PullResistor {
    pub fn to_mask(self) -> u32 {
        let value = match self {
            PullResistor::PullDown => 0b00,
            PullResistor::PullUp => 0b10,
            PullResistor::None => 0b01,
        };

        value << 3
    }
}
