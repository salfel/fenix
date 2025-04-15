use crate::{
    boards::bblack::clock::FuncClock,
    internals::{
        interrupts::{self, Interrupt},
        timer::{current_handler, TimerRegister},
    },
    utils::{nop, wreg},
};

const TIMER_IRQSTATUS: u32 = 0x28;
const TIMER_IRQENABLE_SET: u32 = 0x2C;
const TIMER_CONTROL: u32 = 0x38;
const TIMER_COUNTER: u32 = 0x3C;
const TIMER_LOAD: u32 = 0x40;

const TIMER_INTERVAL: f32 = 31.25;

pub struct Register {
    handlers: [fn(); 6],
}

impl TimerRegister for Register {
    type Timer = Timer;

    fn register(&mut self, timer: Timer, micros: u32, handler: fn()) {
        let reload = self.reload(micros);

        self.handlers[timer as usize] = handler;

        timer.init(reload, Self::interrupt_handler);
    }

    fn current_handler(&self) -> fn() {
        let current_timer = Self::current_timer();

        self.handlers[current_timer as usize]
    }
}

impl Register {
    pub const fn new() -> Self {
        Self { handlers: [nop; 6] }
    }

    fn reload(&self, micros: u32) -> u32 {
        0xFFFF_FFFF - (micros as f32 / TIMER_INTERVAL) as u32
    }

    fn current_timer() -> Timer {
        let interrupt = interrupts::current();

        Timer::try_from(interrupt).unwrap()
    }

    fn interrupt_handler() {
        let current_handler = current_handler();
        let current_timer = Self::current_timer();

        current_timer.handle_irq(current_handler);
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub enum Timer {
    Timer2 = 0,
    Timer3 = 1,
    Timer4 = 2,
    Timer5 = 3,
    Timer6 = 4,
    Timer7 = 5,
}

impl Timer {
    fn init(&self, reload: u32, handler: fn()) {
        self.init_clock();
        self.init_counter(reload);
        self.init_interrupt(handler);

        self.start();
    }

    fn init_clock(&self) {
        let clock: FuncClock = self.into();
        clock.enable();
    }

    fn init_counter(&self, reload: u32) {
        wreg(self.address() + TIMER_LOAD, reload);
        wreg(self.address() + TIMER_COUNTER, reload);
    }

    fn init_interrupt(&self, handler: fn()) {
        self.irq_enable();

        interrupts::enable(self.into(), 0);
        interrupts::register_handler(self.into(), handler);
    }

    fn start(&self) {
        wreg(self.address() + TIMER_CONTROL, 0x3);
    }

    fn irq_enable(&self) {
        wreg(self.address() + TIMER_IRQENABLE_SET, 0x2);
    }

    fn irq_acknowledge(&self) {
        wreg(self.address() + TIMER_IRQSTATUS, 0x2);
    }

    fn handle_irq(&self, handler: fn()) {
        handler();
        self.irq_acknowledge();
    }

    fn address(&self) -> u32 {
        match self {
            Timer::Timer2 => 0x4804_0000,
            Timer::Timer3 => 0x4804_2000,
            Timer::Timer4 => 0x4804_4000,
            Timer::Timer5 => 0x4804_6000,
            Timer::Timer6 => 0x4804_8000,
            Timer::Timer7 => 0x4804_A000,
        }
    }
}

impl TryFrom<Interrupt> for Timer {
    type Error = ();

    fn try_from(interrupt: Interrupt) -> Result<Self, Self::Error> {
        match interrupt {
            Interrupt::TINT2 => Ok(Timer::Timer2),
            Interrupt::TINT3 => Ok(Timer::Timer3),
            Interrupt::TINT4 => Ok(Timer::Timer4),
            Interrupt::TINT5 => Ok(Timer::Timer5),
            Interrupt::TINT6 => Ok(Timer::Timer6),
            Interrupt::TINT7 => Ok(Timer::Timer7),
        }
    }
}

impl From<&Timer> for Interrupt {
    fn from(timer: &Timer) -> Self {
        match timer {
            Timer::Timer2 => Interrupt::TINT2,
            Timer::Timer3 => Interrupt::TINT3,
            Timer::Timer4 => Interrupt::TINT4,
            Timer::Timer5 => Interrupt::TINT5,
            Timer::Timer6 => Interrupt::TINT6,
            Timer::Timer7 => Interrupt::TINT7,
        }
    }
}

impl From<&Timer> for FuncClock {
    fn from(timer: &Timer) -> Self {
        match timer {
            Timer::Timer2 => FuncClock::Timer2,
            Timer::Timer3 => FuncClock::Timer3,
            Timer::Timer4 => FuncClock::Timer4,
            Timer::Timer5 => FuncClock::Timer5,
            Timer::Timer6 => FuncClock::Timer6,
            Timer::Timer7 => FuncClock::Timer7,
        }
    }
}
