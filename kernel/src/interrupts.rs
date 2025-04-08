use libfenix::{noop, read_addr, set_bit, write_addr, INTC};

const INTC_ILR: u32 = 0x100;
const INTC_SIR_IRQ: u32 = 0x40;
const INTC_CONTROL: u32 = 0x48;

pub enum Interrupt {
    I2C2INT = 30,
    TINT2 = 68,
    TINT3 = 69,
    TINT4 = 92,
    TINT5 = 93,
    TINT6 = 94,
    TINT7 = 95,
    GPIOINT1A = 98,
}

impl Interrupt {
    pub fn new(num: u32) -> Option<Self> {
        match num {
            30 => Some(Interrupt::I2C2INT),
            68 => Some(Interrupt::TINT2),
            69 => Some(Interrupt::TINT3),
            92 => Some(Interrupt::TINT4),
            93 => Some(Interrupt::TINT5),
            94 => Some(Interrupt::TINT6),
            95 => Some(Interrupt::TINT7),
            98 => Some(Interrupt::GPIOINT1A),
            _ => None,
        }
    }
}

#[no_mangle]
fn handle_interrupt() {
    let interrupt = current();
    execute(interrupt);
    clear();
}

pub fn enable_interrupt(interrupt: Interrupt, mode: Mode, priority: u8) {
    let interrupt_number = interrupt as u32;

    let addr = INTC + INTC_ILR + (4 * interrupt_number);
    let enable_fiq = match mode {
        Mode::IRQ => 0,
        Mode::FIQ => 1,
    };
    let bank = match InterruptBank::new(interrupt_number) {
        Some(bank) => bank,
        None => return,
    };

    write_addr(addr, enable_fiq | (priority << 2) as u32);
    set_bit(INTC + bank.get_mir() + 4, interrupt_number % 32);
}

static mut INTERRUPT_HANDLERS: &mut [fn(); 128] = &mut [noop; 128];

pub fn register_handler(handler: fn(), interrupt: Interrupt) {
    unsafe {
        INTERRUPT_HANDLERS[interrupt as usize] = handler;
    }
}

pub fn current() -> Option<Interrupt> {
    let num = read_addr(INTC + INTC_SIR_IRQ) & 0x7F;

    Interrupt::new(num)
}

pub fn execute(interrupt: Option<Interrupt>) {
    if let Some(interrupt) = interrupt {
        unsafe { INTERRUPT_HANDLERS[interrupt as usize]() }
    }
}

pub fn clear() {
    write_addr(INTC + INTC_CONTROL, 0x1);
}

pub enum Mode {
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

    fn get_mir(&self) -> u32 {
        match self {
            InterruptBank::Int0 => 0x84,
            InterruptBank::Int1 => 0xA4,
            InterruptBank::Int2 => 0xC4,
            InterruptBank::Int3 => 0xE4,
        }
    }
}
