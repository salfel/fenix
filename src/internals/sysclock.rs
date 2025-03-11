use core::arch::asm;

use super::timer::{self, DmTimer};

pub fn initialize() {
    timer::register_timer(DmTimer::Timer2, 0xFFFF_FFE0, interrupt_handler);
}

static mut SYS_CLOCK: SysClock = SysClock::new();

#[allow(static_mut_refs)]
fn sys_clock() -> &'static mut SysClock {
    unsafe { &mut SYS_CLOCK }
}

pub struct SysClock {
    ticks: u32,
}

impl SysClock {
    const fn new() -> Self {
        SysClock { ticks: 0 }
    }

    fn increment(&mut self) {
        self.ticks += 1;

        if self.ticks % 10 == 0 {
            unsafe { yield_task() };
        }
    }
}

fn interrupt_handler() {
    let sys_clock = sys_clock();

    sys_clock.increment();
}

pub fn millis() -> u32 {
    let sys_clock = sys_clock();
    sys_clock.ticks
}

pub fn wait(ms: u32) {
    let target = millis() + ms;
    loop {
        if millis() > target {
            break;
        } else {
            unsafe {
                asm!("nop");
            }
        }
    }
}

extern "C" {
    fn yield_task();
}
