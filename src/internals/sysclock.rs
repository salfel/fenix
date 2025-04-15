use crate::utils::nop;

use super::timer::{register_timer, Timer};

static mut SYSCLOCK: Sysclock = Sysclock::new();

struct Sysclock {
    ticks: u32,
}

impl Sysclock {
    const fn new() -> Self {
        Sysclock { ticks: 0 }
    }

    fn ticks(&self) -> u32 {
        self.ticks
    }

    fn irq_handler() {
        let sysclock = &raw mut SYSCLOCK;

        unsafe {
            (*sysclock).ticks += 1;
        }
    }
}

impl Default for Sysclock {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn init() {
    register_timer(Timer::Timer2, 1000, Sysclock::irq_handler);
}

pub fn ticks() -> u32 {
    let sysclock = &raw mut SYSCLOCK;
    unsafe { (*sysclock).ticks() }
}

pub fn wait(ms: u32) {
    let current_ticks = ticks();

    loop {
        if ticks() - current_ticks >= ms {
            break;
        }

        // needed to prevent compiler optimizations
        nop();
    }
}
