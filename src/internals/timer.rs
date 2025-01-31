use crate::{
    interrupts::{enable_interrupt, register_handler, Mode},
    peripherals::gpio::{
        self,
        pins::{GPIO1_22, GPIO1_23},
    },
    sys::{write_addr, CM_PER},
};

const TIMER2: u32 = 0x4804_0000;

const CM_PER_L4LS_CLKCTRL: u32 = 0x60;
const CM_PER_TIMER2_CLKCTRL: u32 = 0x80;

const TIMER_CONTROL: u32 = 0x38;
const TIMER_COUNTER: u32 = 0x3C;
const TIMER_LOAD: u32 = 0x40;

const TIMER_IRQ_EOI: u32 = 0x20;
const TIMER_IRQSTATUS: u32 = 0x28;
const TIMER_IRQENABLE_SET: u32 = 0x2C;
const TIMER_IRQENABLE_CLR: u32 = 0x30;

const TIMER_RELOAD_VALUE: u32 = 0xFFFF_FFE0;

const TINT2: u32 = 68;

static mut TIMER: Timer = Timer::new();

#[allow(static_mut_refs)]
pub fn get_timer<'a>() -> &'a mut Timer {
    unsafe { &mut TIMER }
}

pub fn initialize() {
    let timer = get_timer();
    timer.init_clocks();

    timer.stop();
    timer.init_timer();
    timer.init_interrupt();
    timer.start();
}

pub struct Timer {
    counter: u32,
}

impl Timer {
    const fn new() -> Self {
        Timer { counter: 0 }
    }

    fn init_clocks(&self) {
        write_addr(CM_PER + CM_PER_L4LS_CLKCTRL, 0x2);
        write_addr(CM_PER + CM_PER_TIMER2_CLKCTRL, 0x2);
    }

    fn init_timer(&self) {
        write_addr(TIMER2 + TIMER_COUNTER, TIMER_RELOAD_VALUE);
        write_addr(TIMER2 + TIMER_LOAD, TIMER_RELOAD_VALUE);
    }

    fn init_interrupt(&self) {
        write_addr(TIMER2 + TIMER_IRQENABLE_SET, 0x2);

        register_handler(handle_timer_interrupt, TINT2 as usize);
        enable_interrupt(TINT2, Mode::IRQ, 0);
    }

    fn start(&self) {
        write_addr(TIMER2 + TIMER_CONTROL, 0x2);
    }

    fn stop(&self) {
        write_addr(TIMER2 + TIMER_CONTROL, 0x0);
    }

    fn irq_acknowledge(&self) {
        write_addr(TIMER2 + TIMER_IRQ_EOI, 0x0);
    }

    fn increment(&mut self) {
        self.counter += 1;
    }
}

fn handle_timer_interrupt() {
    let timer = get_timer();

    timer.irq_acknowledge();
    gpio::write(GPIO1_23, true);
}
