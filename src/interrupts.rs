use crate::{
    interfaces::gpio,
    sys::{read_addr, set_bit, write_addr, INTC},
};

const INTC_ILR: u32 = 0x100;
const INTC_SIR_IRQ: u32 = 0x40;
const INTC_CONTROL: u32 = 0x48;

const GPIOINT1A: u32 = 98;
const GPIOINT1B: u32 = 99;

pub fn initialize() {
    // reset all interrupts
    for bank in InterruptBank::all() {
        write_addr(INTC + bank.get_mir() + 4, 0xFFFFFFFF);
    }

    //enable_interrupt(GPIOINT1A, Mode::IRQ, 0);
    //enable_interrupt(GPIOINT1B, Mode::IRQ, 0);
}

fn enable_interrupt(n: u32, mode: Mode, priority: u8) {
    let addr = INTC + INTC_ILR + (4 * n);
    let enable_fiq = match mode {
        Mode::IRQ => 0,
        Mode::FIQ => 1,
    };
    let bank = match InterruptBank::new(n) {
        Some(bank) => bank,
        None => return,
    };

    write_addr(addr, enable_fiq | (priority << 2) as u32);
    set_bit(INTC + bank.get_mir() + 8, n);
}

pub struct Interrupt {
    number: u8,
}

impl Interrupt {
    pub fn get_current() -> Self {
        let value = read_addr(INTC + INTC_SIR_IRQ);
        Interrupt {
            number: (value & 0b111_1111) as u8,
        }
    }

    pub fn execute(self) {
        match self.number as u32 {
            GPIOINT1A | GPIOINT1B => {
                gpio::write(21, true);
            }
            _ => {}
        }

        set_bit(INTC + INTC_CONTROL, 1);
    }
}

enum Mode {
    IRQ,
    FIQ,
}

#[repr(u32)]
enum InterruptBank {
    Int0,
    Int1,
    Int2,
    Int3,
}

impl InterruptBank {
    pub fn new(n: u32) -> Option<InterruptBank> {
        match n {
            0..32 => Some(InterruptBank::Int0),
            32..64 => Some(InterruptBank::Int1),
            64..96 => Some(InterruptBank::Int2),
            96..128 => Some(InterruptBank::Int3),
            _ => None,
        }
    }

    pub fn all<'a>() -> &'a [InterruptBank] {
        &[
            InterruptBank::Int0,
            InterruptBank::Int1,
            InterruptBank::Int2,
            InterruptBank::Int3,
        ]
    }

    fn get_mir(&self) -> u32 {
        match self {
            InterruptBank::Int0 => 0x84,
            InterruptBank::Int1 => 0xA4,
            InterruptBank::Int2 => 0xC4,
            InterruptBank::Int3 => 0xE4,
        }
    }
}
