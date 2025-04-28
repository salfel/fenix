use super::timer::{self, DmTimer};

pub fn initialize() {
    timer::register_timer(DmTimer::Timer2, 0xFFFF_FFE0, interrupt_handler);
}

static mut SYS_CLOCK: u32 = 0;

fn interrupt_handler() {
    unsafe { SYS_CLOCK += 1 };

    if unsafe { SYS_CLOCK } % 10 == 0 {
        unsafe { yield_task() };
    }
}

pub fn millis() -> u32 {
    unsafe { SYS_CLOCK }
}

extern "C" {
    fn yield_task();
}
