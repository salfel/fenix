use super::task::{Task, TaskState};

const MAX_TASKS: usize = 8;

static mut TASK_MANAGER: TaskManager = TaskManager::new();

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
                let new_task = Task::new(id as u32, entry_point as usize, priority);

                *task = Some(new_task);

                if let Some(task) = task {
                    task.setup_stack();
                }

                break;
            }
        }

        Err(TaskCreationError)
    }

    fn cycle(&mut self) {
        let mut highest_priority = None;

        for task in self
            .tasks
            .iter()
            .flatten()
            .filter(|task| task.state.executable())
        {
            match highest_priority {
                None => highest_priority = Some((task.id, task.priority)),
                Some((_, priority)) => {
                    if task.priority < priority {
                        highest_priority = Some((task.id, task.priority));
                    }
                }
            }
        }

        let task_id = match highest_priority {
            Some((id, _)) => id,
            None => return,
        };

        self.current_task = task_id as usize;
        let task = self.current().unwrap();

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

        self.cycle();
    }

    fn yield_context(&mut self, sp: usize, pc: usize, until: u32) {
        let task = self.current();
        if let Some(task) = task {
            task.sp = sp;
            task.pc = pc;
            task.state = TaskState::Blocked(until);
        }

        self.cycle();
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

pub fn cycle() {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe {
        (*task_manager).cycle();
    }
}

pub fn create_task(entry_point: fn(), priority: u8) -> Result<(), TaskCreationError> {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe { (*task_manager).create_task(entry_point, priority) }
}

extern "C" {
    fn switch_context(sp: usize, pc: usize);
    fn restore_context(sp: usize, pc: usize);
}
