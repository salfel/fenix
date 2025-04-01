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

    /// Attempts to convert a `TrapFrame` into a corresponding `Syscall` variant.
    /// 
    /// This method examines the value in the `r12` register to determine the syscall type:
    /// - If `r12` is 0, it returns a `Syscall::Exit`.
    /// - If `r12` is 1, it returns a `Syscall::Yield` with the stack pointer from `r0`, the program counter from `r1`, and an optional resume time derived from `r2` (where a value of 0 yields `None`).
    /// - For any other value, it returns a `SyscallError`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::convert::TryInto;
    /// #
    /// # // Mock definitions for demonstration purposes
    /// # #[derive(Debug)]
    /// # pub struct TrapFrame { pub r0: u32, pub r1: u32, pub r2: u32, pub r3: u32, pub r12: u32 }
    /// #
    /// # #[derive(Debug, PartialEq)]
    /// # pub enum Syscall {
    /// #     Exit,
    /// #     Yield { sp: u32, pc: u32, until: Option<u32> },
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # pub struct SyscallError;
    /// #
    /// # impl TryInto<Syscall> for &TrapFrame {
    /// #     type Error = SyscallError;
    /// #
    /// #     fn try_into(self) -> Result<Syscall, Self::Error> {
    /// #         match self.r12 {
    /// #             0 => Ok(Syscall::Exit),
    /// #             1 => Ok(Syscall::Yield {
    /// #                 sp: self.r0,
    /// #                 pc: self.r1,
    /// #                 until: match self.r2 {
    /// #                     0 => None,
    /// #                     x => Some(x),
    /// #                 },
    /// #             }),
    /// #             _ => Err(SyscallError),
    /// #         }
    /// #     }
    /// # }
    ///
    /// let frame = TrapFrame { r0: 100, r1: 200, r2: 0, r3: 0, r12: 1 };
    /// let syscall: Syscall = (&frame).try_into().unwrap();
    /// match syscall {
    ///     Syscall::Yield { sp, pc, until } => {
    ///         assert_eq!(sp, 100);
    ///         assert_eq!(pc, 200);
    ///         assert!(until.is_none());
    ///     },
    ///     _ => panic!("Unexpected syscall variant"),
    /// }
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
/// Handles a software interrupt by converting a trap frame into a corresponding syscall and executing the requested task operation.
///
/// This function attempts to convert the provided trap frame into a syscall using the `TryInto<Syscall>` trait. If the conversion fails,
/// the function returns `false`. On a successful conversion, it matches on the syscall variant:
///
/// - **Exit:** Terminates the current task (if one is active) and cycles the scheduler.
/// - **Yield (without an `until` value):** Updates the current task’s context (program counter and stack pointer), sets the task state to stored,
///   and cycles the scheduler.
/// - **Yield (with an `until` value):** Updates the task’s context and sets the task state to waiting until the specified time, then cycles the scheduler.
///
/// Returns `true` if the syscall was handled successfully, or `false` if the trap frame did not translate into a valid syscall.
///
/// # Examples
///
/// ```
/// // Construct a trap frame with an invalid syscall indicator (e.g., r12 set to an undefined value).
/// let frame = TrapFrame { r0: 0, r1: 0, r2: 0, r3: 0, r12: 99 };
/// let handled = unsafe { swi_handler(&frame) };
/// assert!(!handled);
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
