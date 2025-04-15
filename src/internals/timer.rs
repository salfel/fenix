#[cfg(feature = "bblack")]
use crate::boards::bblack::internals::timer::{Register, self};

pub use timer::Timer;

pub trait TimerRegister {
    type Timer;

    fn register(&mut self, timer: Self::Timer, micros: u32, callback: fn());

    fn current_handler(&self) -> fn();
}

static mut TIMER_REGISTER: Register = Register::new();

#[allow(static_mut_refs)]
pub(crate) fn current_handler() -> fn() {
    unsafe {
        TIMER_REGISTER.current_handler()
    }
}

#[allow(static_mut_refs)]
pub fn register_timer(timer: Timer, micros: u32, handler: fn()) {
    unsafe {
        TIMER_REGISTER.register(timer, micros, handler);
    }
}
