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

    /// Attempts to convert a reference to a TrapFrame into a corresponding Syscall.
    ///
    /// The conversion inspects the `r12` field:
    /// - If `r12` is `0`, the frame represents an exit syscall.
    /// - If `r12` is `1`, the frame represents a yield syscall. The yield syscall extracts:
    ///   - `sp` from `r0`
    ///   - `pc` from `r1`
    ///   - an optional `until` value from `r2` (interpreted as `None` if `r2` is `0`)
    /// - Any other value of `r12` results in a conversion error, yielding a `SyscallError`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a TrapFrame representing a yield syscall with no wait time
    /// let frame = TrapFrame { r0: 100, r1: 200, r2: 0, r3: 0, r12: 1 };
    /// let syscall = (&frame).try_into().expect("Failed to convert TrapFrame");
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
/// Handles a software interrupt by converting the provided trap frame into a system call and executing the corresponding operation.
/// 
/// This function attempts to convert the given `TrapFrame` (which contains the CPU register state at the time of the interrupt) into a recognized system call.
/// If the conversion fails, it returns `false`. Upon successful conversion, it processes the system call as follows:
/// 
/// - **Exit:** Terminates the current task and cycles the scheduler.
/// - **Yield without a wait time:** Updates the current task's context with the supplied stack pointer and program counter, marks it as stored, and cycles the scheduler.
/// - **Yield with a wait time:** Updates the current task's context with the supplied stack pointer and program counter, marks it as waiting until the specified time, and cycles the scheduler.
/// 
/// Returns `true` if a valid system call was processed, or `false` if the trap frame did not correspond to a recognized system call.
/// 
/// # Examples
/// 
/// ```
/// // Example trap frame representing an Exit syscall (r12 == 0).
/// let frame = TrapFrame { r0: 0, r1: 0, r2: 0, r3: 0, r12: 0 };
/// 
/// // The swi_handler should process the exit syscall and return true.
/// assert!(swi_handler(&frame));
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
