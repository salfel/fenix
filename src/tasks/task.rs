use crate::internals::sysclock::ticks;

const TASK_STACK_SIZE: usize = 1024;

pub struct Task {
    pub id: u32,
    pub sp: usize,
    pub pc: usize,
    pub priority: u8,
    pub stack: [u8; TASK_STACK_SIZE],
    pub state: TaskState,
}

impl Task {
    pub fn new(id: u32, pc: usize, priority: u8) -> Self {
        Task {
            id,
            sp: 0,
            pc,
            priority,
            state: TaskState::Ready,
            stack: [0; TASK_STACK_SIZE],
        }
    }

    pub fn setup_stack(&mut self) {
        self.sp = self.stack.as_ptr() as usize + TASK_STACK_SIZE;
    }
}

pub enum TaskState {
    Ready,
    Stored,
    Running,
    Blocked(u32),
}

impl TaskState {
    pub fn executable(&self) -> bool {
        match self {
            TaskState::Ready => true,
            TaskState::Stored => true,
            TaskState::Running => false,
            TaskState::Blocked(until) => ticks() >= *until,
        }
    }
}

