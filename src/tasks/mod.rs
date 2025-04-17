use core::arch::global_asm;

use crate::internals::sysclock::ticks;

const TASK_STACK_SIZE: usize = 1024;
const MAX_TASKS: usize = 8;

global_asm!(include_str!("tasks.asm"));

static mut TASK_MANAGER: TaskManager = TaskManager::new();

pub static mut EXECUTING: bool = false;

pub fn executing() -> bool {
    unsafe { EXECUTING }
}

pub fn set_executing(value: bool) {
    unsafe { EXECUTING = value }
}

pub struct TaskCreationError;

struct TaskManager {
    tasks: [Option<Task>; MAX_TASKS],
    current_task: usize,
}

impl TaskManager {
    const fn new() -> Self {
        TaskManager {
            tasks: [const { None }; MAX_TASKS],
            current_task: 0,
        }
    }

    fn task(&mut self, index: usize) -> Option<&mut Task> {
        let task = &mut self.tasks[index];

        if let Some(task) = task {
            Some(task)
        } else {
            None
        }
    }

    fn current(&mut self) -> Option<&mut Task> {
        self.task(self.current_task)
    }

    fn create_task(&mut self, entry_point: fn(), priority: u8) -> Result<(), TaskCreationError> {
        for (id, task) in self.tasks.iter_mut().enumerate() {
            if task.is_none() {
                let mut new_task = Task::new(id as u32, entry_point as usize, priority);
                new_task.setup_stack();

                *task = Some(new_task);
                break;
            }
        }

        Err(TaskCreationError)
    }

    fn cycle(&mut self) {
        let mut highest_priority = None;

        self.tasks.iter().for_each(|task| {
            if let Some(task) = task {
                if !task.state.executable() {
                    return;
                }

                match highest_priority {
                    None => highest_priority = Some((task.id, task.priority)),
                    Some((_, priority)) => {
                        if priority < task.priority {
                            highest_priority = Some((task.id, task.priority));
                        }
                    }
                }
            }
        });

        let task_id = match highest_priority {
            Some((id, _)) => id,
            None => return,
        };

        self.current_task = task_id as usize;
        let task = self.current().unwrap();

        set_executing(true);

        match task.state {
            TaskState::Ready => {
                task.state = TaskState::Running;
                unsafe {
                    switch_context(task.sp, task.pc);
                }
            }
            TaskState::Stored => {
                task.state = TaskState::Running;
                unsafe {
                    restore_context(task.sp, task.pc);
                }
            }
            TaskState::Blocked(_) => {
                task.state = TaskState::Running;
                unsafe {
                    restore_context(task.sp, task.pc);
                }
            }
            _ => {}
        }
    }

    fn save_context(&mut self, sp: usize, pc: usize) {
        let task = self.current();
        if let Some(task) = task {
            task.sp = sp;
            task.pc = pc;
            task.state = TaskState::Stored;
        }

        set_executing(false);

        kernel_loop();
    }

    fn yield_context(&mut self, sp: usize, pc: usize, until: u32) {
        let task = self.current();
        if let Some(task) = task {
            task.sp = sp;
            task.pc = pc;
            task.state = TaskState::Blocked(until);
        }

        set_executing(false);

        kernel_loop();
    }
}

struct Task {
    id: u32,
    sp: usize,
    pc: usize,
    priority: u8,
    stack: [u8; TASK_STACK_SIZE],
    state: TaskState,
}

impl Task {
    fn new(id: u32, pc: usize, priority: u8) -> Self {
        Task {
            id,
            sp: 0,
            pc,
            priority,
            state: TaskState::Ready,
            stack: [0; TASK_STACK_SIZE],
        }
    }

    fn setup_stack(&mut self) {
        self.sp = self.stack.as_ptr() as usize + TASK_STACK_SIZE - 4;
    }
}

enum TaskState {
    Ready,
    Stored,
    Running,
    Blocked(u32),
}

impl TaskState {
    fn executable(&self) -> bool {
        match self {
            TaskState::Ready => true,
            TaskState::Stored => true,
            TaskState::Running => false,
            TaskState::Blocked(until) => ticks() >= *until,
        }
    }
}

#[no_mangle]
fn save_context(sp: usize, pc: usize) {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe {
        (*task_manager).save_context(sp, pc);
    }
}

#[no_mangle]
fn yield_context(sp: usize, pc: usize, until: u32) {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe {
        (*task_manager).yield_context(sp, pc, until);
    }
}

pub fn kernel_loop() -> ! {
    let task_manager = &raw mut TASK_MANAGER;

    loop {
        unsafe {
            (*task_manager).cycle();
        }
    }
}

pub fn create_task(entry_point: fn(), priority: u8) -> Result<(), TaskCreationError> {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe { (*task_manager).create_task(entry_point, priority) }
}

extern "C" {
    pub fn yield_task(until: u32);
    fn switch_context(sp: usize, pc: usize);
    fn restore_context(sp: usize, pc: usize);
}
