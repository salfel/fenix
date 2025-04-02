use crate::sync::mutex::Mutex;

use super::timer::{self, DmTimer};

pub fn initialize() {
    timer::register_timer(DmTimer::Timer2, 0xFFFF_FFE0, interrupt_handler);
}

pub static SYS_CLOCK: Mutex<u32> = Mutex::new(0);

fn interrupt_handler() {
    let mut ticks = SYS_CLOCK.lock();
    *ticks += 1;

    if *ticks % 10 == 0 {
        unsafe { yield_task() };
    }
}

pub fn millis() -> u32 {
    unsafe { unsafe_millis() }
}

#[no_mangle]
pub fn wait(ms: u32) {
    unsafe {
        wait_store(millis() + ms);
    }
}

extern "C" {
    fn unsafe_millis() -> u32;
    fn yield_task();
    fn wait_store(ms: u32);
}
