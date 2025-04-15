use crate::{
    internals::interrupts::InterruptRegister,
    utils::{rreg, wbit, wreg},
};

const INTC: u32 = 0x48200000;

const INTC_ILR: u32 = 0x100;
const INTC_SIR_IRQ: u32 = 0x40;
const INTC_CONTROL: u32 = 0x48;

const MIR_CLR_OFFSET: u32 = 0x4;

pub struct Register {
    handlers: [fn(); 128],
}

impl InterruptRegister for Register {
    fn enable(&self, interrupt: Interrupt, priority: u8) {
        let interrupt_number = interrupt as u32;

        let addr = INTC + INTC_ILR + (4 * interrupt_number);
        let bank = match InterruptBank::new(interrupt_number) {
            Some(bank) => bank,
            None => return,
        };
        let mode = Mode::IRQ;

        wreg(addr, mode as u32 | (priority << 2) as u32);
        wbit(
            INTC + bank.get_mir() + MIR_CLR_OFFSET,
            interrupt_number % 32,
            true,
        );
    }

    fn register_handler(&mut self, interrupt: Interrupt, handler: fn()) {
        self.handlers[interrupt as usize] = handler;
    }

    fn current(&self) -> Interrupt {
        let num = rreg(INTC + INTC_SIR_IRQ) & 0x7F;

        Interrupt::new(num)
    }

    fn handle_interrupt(&self) {
        let interrupt = self.current();
        self.execute(interrupt);
        self.clear();
    }
}

impl Register {
    pub const fn new() -> Self {
        Register {
            handlers: [noop; 128],
        }
    }

    fn execute(&self, interrupt: Interrupt) {
        self.handlers[interrupt as usize]()
    }

    fn clear(&self) {
        wreg(INTC + INTC_CONTROL, 0x1);
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Interrupt {
    TINT2 = 68,
    TINT3 = 69,
    TINT4 = 92,
    TINT5 = 93,
    TINT6 = 94,
    TINT7 = 95,
}

impl Interrupt {
    pub fn new(num: u32) -> Self {
        match num {
            68 => Interrupt::TINT2,
            69 => Interrupt::TINT3,
            92 => Interrupt::TINT4,
            93 => Interrupt::TINT5,
            94 => Interrupt::TINT6,
            95 => Interrupt::TINT7,
            _ => panic!(),
        }
    }
}

pub enum Mode {
    IRQ = 0,
    FIQ = 1,
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

    fn get_mir(&self) -> u32 {
        match self {
            InterruptBank::Int0 => 0x84,
            InterruptBank::Int1 => 0xA4,
            InterruptBank::Int2 => 0xC4,
            InterruptBank::Int3 => 0xE4,
        }
    }
}

fn noop() {}
