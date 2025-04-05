use libfenix::write_addr;

const CONTROL_MODULE_BASE: u32 = 0x44E10000;
const CONF_GPMC_BEN1: u32 = 0x878; // GPIO1_28
const CONF_GMPC_A0: u32 = 0x840; // GPIO1_16
const CONF_GMPC_A1: u32 = 0x844; // GPIO1_17
const CONF_GMPC_A2: u32 = 0x848; // GPIO1_18
const CONF_GMPC_A3: u32 = 0x84C; // GPIO1_19

pub fn configure() {
    set_pin_mode(CONF_GPMC_BEN1, 7, true, PullResistor::PullDown);
    set_pin_mode(CONF_GMPC_A0, 7, true, PullResistor::PullDown);
    set_pin_mode(CONF_GMPC_A1, 7, true, PullResistor::PullDown);
    set_pin_mode(CONF_GMPC_A2, 7, true, PullResistor::PullDown);
    set_pin_mode(CONF_GMPC_A3, 7, true, PullResistor::PullDown);
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
