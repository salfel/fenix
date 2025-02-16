use core::arch::asm;

use crate::{interrupts, sys::write_addr};

use super::clock::{self, FuncClock};

const TIMER_IRQ_EOI: u32 = 0x20;
const TIMER_IRQSTATUS: u32 = 0x28;
const TIMER_IRQENABLE_SET: u32 = 0x2C;
const TIMER_IRQENABLE_CLR: u32 = 0x30;
const TIMER_CONTROL: u32 = 0x38;
const TIMER_COUNTER: u32 = 0x3C;
const TIMER_LOAD: u32 = 0x40;

pub const DMTIMER2: u32 = 0x4804_0000;

const CLOCK_RELOAD_VALUE: u32 = 0xFFFF_FFE0;

pub fn initialize() {
    let timer = get_timer();

    timer.stop();
    timer.init();
    timer.init_clock();
    timer.init_interrupts();
    timer.start();
}

static mut TIMER: Timer = Timer::new();

#[allow(static_mut_refs)]
pub fn get_timer() -> &'static mut Timer {
    unsafe { &mut TIMER }
}

pub struct Timer {
    ticks: u32,
}

impl Timer {
    const fn new() -> Self {
        Timer { ticks: 0 }
    }

    fn init_clock(&self) {
        clock::enable(FuncClock::Timer2);
    }

    fn init(&self) {
        write_addr(DMTIMER2 + TIMER_LOAD, CLOCK_RELOAD_VALUE);
        write_addr(DMTIMER2 + TIMER_COUNTER, CLOCK_RELOAD_VALUE);
    }

    fn init_interrupts(&self) {
        self.irq_enable();

        interrupts::register_handler(handle_timer_irq, 68);
        interrupts::enable_interrupt(68, interrupts::Mode::IRQ, 0);
    }

    fn start(&self) {
        write_addr(DMTIMER2 + TIMER_CONTROL, 0x3);
    }

    fn stop(&self) {
        write_addr(DMTIMER2 + TIMER_CONTROL, 0x0);
    }

    fn reset(&self) {
        write_addr(DMTIMER2 + TIMER_COUNTER, CLOCK_RELOAD_VALUE);
    }

    fn irq_enable(&self) {
        write_addr(DMTIMER2 + TIMER_IRQENABLE_SET, 0x2);
    }

    fn irq_disable(&self) {
        write_addr(DMTIMER2 + TIMER_IRQENABLE_CLR, 0x2);
    }

    fn irq_acknowledge(&self) {
        write_addr(DMTIMER2 + TIMER_IRQ_EOI, 0x0);
        write_addr(DMTIMER2 + TIMER_IRQSTATUS, 0x7);
    }

    fn increment(&mut self) {
        self.ticks += 1;
    }

    fn elapsed(&self) -> u32 {
        self.ticks
    }
}


fn handle_timer_irq() {
    let timer = get_timer();

    timer.irq_disable();
    timer.stop();
    timer.irq_acknowledge();
    timer.reset();
    timer.increment();
    timer.irq_enable();
    timer.start();
}


pub fn millis() -> u32 {
    let timer = get_timer();
    timer.elapsed()
}

pub fn wait_ms(ms: u32) {
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
