use core::convert::TryInto;

use crate::internals::tasks::{scheduler, TaskState};

enum Syscall {
    Exit,
    Yield {
        sp: u32,
        pc: u32,
        until: Option<u32>,
    },
}

struct SyscallError {}

impl TryInto<Syscall> for &TrapFrame {
    type Error = SyscallError;

    /// Converts a trap frame into a corresponding system call.
    /// 
    /// This function examines the trap frame's registers to determine the desired system call:
    /// 
    /// - If `r12` is `0`, it returns `Syscall::Exit`.
    /// - If `r12` is `1`, it returns `Syscall::Yield` with:
    ///   - `sp` taken from `r0`,
    ///   - `pc` taken from `r1`, and
    ///   - an optional `until` value from `r2` (where a value of `0` indicates no `until` value).
    /// - Any other value results in an error (`SyscallError`).
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Convert a trap frame representing a yield syscall without a waiting value.
    /// # use your_module::{TrapFrame, Syscall};
    /// let frame = TrapFrame { r0: 42, r1: 84, r2: 0, r3: 0, r12: 1 };
    /// let syscall = (&frame).try_into();
    /// assert_eq!(syscall, Ok(Syscall::Yield { sp: 42, pc: 84, until: None }));
    /// ```
    fn try_into(self) -> Result<Syscall, Self::Error> {
        match self.r12 {
            0 => Ok(Syscall::Exit),
            1 => Ok(Syscall::Yield {
                sp: self.r0,
                pc: self.r1,
                until: match self.r2 {
                    0 => None,
                    until => Some(until),
                },
            }),
            _ => Err(SyscallError {}),
        }
    }
}

#[repr(C)]
struct TrapFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
}

#[no_mangle]
/// Handles a software interrupt by converting a trap frame into a system call and processing the resulting action.
///
/// The function attempts to convert the provided trap frame into a system call. If the conversion fails,
/// it returns `false`. On a successful conversion, it distinguishes between two system call types:
///
/// - **Exit**: Terminates the current task (if one exists) and cycles the scheduler.
/// - **Yield**: Updates the current task’s context with the provided stack pointer and program counter.
///   - If no `until` value is provided, the task state is set to stored.
///   - If an `until` value is provided, the task state is set to waiting until that time.
///
/// In every valid case, the scheduler is cycled and the function returns `true`.
///
/// # Examples
///
/// ```
/// // Assume `TrapFrame` and `swi_handler` are imported from the kernel module.
///
/// // Example: Create a trap frame configured to trigger an exit system call.
/// let frame = TrapFrame {
///     r0: 0,
///     r1: 0,
///     r2: 0,
///     r3: 0,
///     r12: 0, // Indicates Syscall::Exit
/// };
///
/// let handled = swi_handler(&frame);
/// assert!(handled);
/// ```
extern "C" fn swi_handler(frame: &TrapFrame) -> bool {
    let syscall: Syscall = match frame.try_into() {
        Ok(syscall) => syscall,
        Err(_) => return false,
    };

    match syscall {
        Syscall::Exit => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.terminate();
            }

            scheduler.cycle();

            true
        }
        Syscall::Yield {
            sp,
            pc,
            until: None,
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Stored;
            }

            scheduler.cycle();

            true
        }
        Syscall::Yield {
            sp,
            pc,
            until: Some(until),
        } => {
            let scheduler = scheduler();
            if let Some(task) = scheduler.current() {
                task.context.pc = pc;
                task.context.sp = sp;
                task.state = TaskState::Waiting { until };
            }

            scheduler.cycle();

            true
        }
    }
}

#[no_mangle]
pub fn kernel_loop() {
    loop {
        let scheduler = scheduler();
        scheduler.switch();
    }
}
