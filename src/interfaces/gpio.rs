use crate::{
    interrupts::{self, register_handler, Mode},
    sys::{clear_bit, noop, read_addr, read_bit, set_bit, write_addr, CM_PER, GPIO1},
};

const CM_PER_GPIO1_CLKCTRL: u32 = 0xAC;

const GPIO_OE: u32 = 0x134;
const GPIO_DATAIN: u32 = 0x138;
const GPIO_DATAOUT: u32 = 0x13C;

pub const GPIO_IRQSTATUS_RAW_0: u32 = 0x24;
pub const GPIO_IRQSTATUS_0: u32 = 0x2C;
const GPIO_IRQSTATUS_SET0: u32 = 0x34;

const GPIOINT1A: u32 = 98;

pub fn pin_mode(pin: u32, mode: GpioMode) {
    match mode {
        GpioMode::Input => {
            set_bit(GPIO1 + GPIO_OE, pin);
        }
        GpioMode::Output => {
            clear_bit(GPIO1 + GPIO_OE, pin);
        }
    }
}

pub fn write(pin: u32, value: bool) {
    if value {
        set_bit(GPIO1 + GPIO_DATAOUT, pin);
    } else {
        clear_bit(GPIO1 + GPIO_DATAOUT, pin);
    }
}

pub fn read(pin: u32) -> bool {
    read_bit(GPIO1 + GPIO_DATAIN, pin)
}

pub fn initialize() {
    write_addr(CM_PER + CM_PER_GPIO1_CLKCTRL, 2);
    interrupts::enable_interrupt(GPIOINT1A, Mode::IRQ, 0);
    register_handler(handle_interrupts, GPIOINT1A as usize);
}

static mut GPIO_INTERRUPT_HANDLERS: [fn(); 32] = [noop; 32];

fn handle_interrupts() {
    let irq_raw = read_addr(GPIO1 + GPIO_IRQSTATUS_RAW_0);
    let number = irq_raw.trailing_zeros();

    unsafe {
        GPIO_INTERRUPT_HANDLERS[number as usize]();
    }

    write_addr(GPIO1 + GPIO_IRQSTATUS_0, 1 << number);
}

pub fn enable_interrupt(pin: u32, interrupt: GpioInterrupt, handler: fn()) {
    unsafe {
        GPIO_INTERRUPT_HANDLERS[pin as usize] = handler;
    }

    set_bit(GPIO1 + GPIO_IRQSTATUS_SET0, pin);

    set_bit(GPIO1 + interrupt as u32, pin);
}

pub enum GpioMode {
    Input,
    Output,
}

#[repr(u32)]
pub enum GpioInterrupt {
    Level0 = 0x140,
    Level1 = 0x144,
    Rising = 0x148,
    Falling = 0x14C,
}
