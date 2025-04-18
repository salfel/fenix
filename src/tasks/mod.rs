use core::arch::global_asm;

pub(super) mod scheduler;
pub(super) mod task;

pub use scheduler::{create_task, cycle};

global_asm!(include_str!("tasks.S"));
