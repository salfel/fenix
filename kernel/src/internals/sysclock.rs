use crate::sync::mutex::Mutex;

use super::timer::{self, DmTimer};

pub fn initialize() {
    timer::register_timer(DmTimer::Timer2, 0xFFFF_FFE0, interrupt_handler);
}

pub(crate) static SYS_CLOCK: Mutex<u32> = Mutex::new(0);

fn interrupt_handler() {
    let mut ticks = SYS_CLOCK.lock();
    *ticks += 1;

    if *ticks % 10 == 0 {
        unsafe { yield_task() };
    }
}

pub fn millis() -> u32 {
    *SYS_CLOCK.lock()
}

extern "C" {
    fn yield_task();
}
