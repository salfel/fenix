use crate::{boards::bblack::clock::FuncClock, utils::wreg};

use super::interrupts::{self, Interrupt};

const TIMER_IRQSTATUS: u32 = 0x28;
const TIMER_IRQENABLE_SET: u32 = 0x2C;
const TIMER_IRQENABLE_CLR: u32 = 0x30;
const TIMER_CONTROL: u32 = 0x38;
const TIMER_COUNTER: u32 = 0x3C;
const TIMER_LOAD: u32 = 0x40;

static mut TIMERS: &mut [Option<Timer>; 7] = &mut [const { None }; 7];

pub fn register_timer(dm_timer: DmTimer, reload: u32, handler: fn()) {
    let timer = Timer::new(dm_timer, reload, handler);
    unsafe { TIMERS[dm_timer as usize] = Some(timer) }
}

pub fn get_timer(interrupt: Interrupt) -> &'static Option<Timer> {
    let dm_timer = match DmTimer::try_new(interrupt) {
        Some(dm_timer) => dm_timer,
        None => return &None,
    };

    unsafe { &TIMERS[dm_timer as usize] }
}

pub struct Timer {
    timer: DmTimer,
    reload: u32,
    handler: fn(),
}

impl Timer {
    fn new(timer: DmTimer, reload: u32, handler: fn()) -> Self {
        let timer = Timer {
            timer,
            reload,
            handler,
        };

        timer.init_clock();
        timer.init_counter();
        timer.init_interrupt();

        timer.start();

        timer
    }

    fn init_clock(&self) {
        self.timer.clock().enable();
    }

    fn init_counter(&self) {
        wreg(self.timer.address() + TIMER_LOAD, self.reload);
        wreg(self.timer.address() + TIMER_COUNTER, self.reload);
    }

    fn init_interrupt(&self) {
        self.irq_enable();

        interrupts::enable(self.timer.interrupt(), 0);
        interrupts::register_handler(self.timer.interrupt(), Self::handle_timer_irq);
    }

    fn start(&self) {
        wreg(self.timer.address() + TIMER_CONTROL, 0x3);
    }

    fn irq_enable(&self) {
        wreg(self.timer.address() + TIMER_IRQENABLE_SET, 0x2);
    }

    fn irq_disable(&self) {
        wreg(self.timer.address() + TIMER_IRQENABLE_CLR, 0x2);
    }

    fn irq_acknowledge(&self) {
        wreg(self.timer.address() + TIMER_IRQSTATUS, 0x2);
    }

    fn handle_timer_irq() {
        let interrupt = interrupts::current();

        let timer = get_timer(interrupt);

        if let Some(timer) = timer {
            timer.irq_disable();
            timer.irq_acknowledge();
            (timer.handler)();
            timer.irq_enable();
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum DmTimer {
    Timer2 = 2,
    Timer3 = 3,
    Timer4 = 4,
    Timer5 = 5,
    Timer6 = 6,
    Timer7 = 7,
}

impl DmTimer {
    fn try_new(interrupt: Interrupt) -> Option<Self> {
        match interrupt {
            Interrupt::TINT2 => Some(DmTimer::Timer2),
            Interrupt::TINT3 => Some(DmTimer::Timer3),
            Interrupt::TINT4 => Some(DmTimer::Timer4),
            Interrupt::TINT5 => Some(DmTimer::Timer5),
            Interrupt::TINT6 => Some(DmTimer::Timer6),
            Interrupt::TINT7 => Some(DmTimer::Timer7),
        }
    }

    fn clock(&self) -> FuncClock {
        match self {
            DmTimer::Timer2 => FuncClock::Timer2,
            DmTimer::Timer3 => FuncClock::Timer3,
            DmTimer::Timer4 => FuncClock::Timer4,
            DmTimer::Timer5 => FuncClock::Timer5,
            DmTimer::Timer6 => FuncClock::Timer6,
            DmTimer::Timer7 => FuncClock::Timer7,
        }
    }

    fn address(&self) -> u32 {
        match self {
            DmTimer::Timer2 => 0x4804_0000,
            DmTimer::Timer3 => 0x4804_2000,
            DmTimer::Timer4 => 0x4804_4000,
            DmTimer::Timer5 => 0x4804_6000,
            DmTimer::Timer6 => 0x4804_8000,
            DmTimer::Timer7 => 0x4804_A000,
        }
    }

    fn interrupt(&self) -> Interrupt {
        match self {
            DmTimer::Timer2 => Interrupt::TINT2,
            DmTimer::Timer3 => Interrupt::TINT3,
            DmTimer::Timer4 => Interrupt::TINT4,
            DmTimer::Timer5 => Interrupt::TINT5,
            DmTimer::Timer6 => Interrupt::TINT6,
            DmTimer::Timer7 => Interrupt::TINT7,
        }
    }
}
