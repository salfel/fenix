use core::arch::global_asm;

const TASK_STACK_SIZE: usize = 1024;
const MAX_TASKS: usize = 8;

global_asm!(include_str!("tasks.asm"));

static mut TASK_MANAGER: TaskManager = TaskManager::new();

struct TaskCreationError;

struct TaskManager {
    tasks: [Option<Task>; MAX_TASKS],
    current_task: usize,
}

impl TaskManager {
    const fn new() -> Self {
        let mut tasks = [const { None }; MAX_TASKS];
        tasks[0] = Some(Task {
            stack: [0; TASK_STACK_SIZE],
            sp: 0,
            pc: 0,
            priority: 0,
            state: TaskState::Ready,
        });

        TaskManager {
            tasks,
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

    fn create_task(&mut self, new_task: Task) -> Result<(), TaskCreationError> {
        for task in self.tasks.iter_mut() {
            if task.is_none() {
                *task = Some(new_task);
                return Ok(());
            }
        }

        Err(TaskCreationError)
    }

    fn save_context(&mut self, sp: u32, pc: u32) {
        let task = self.current();
        if let Some(task) = task {
            task.sp = sp;
            task.pc = pc;
        }

        self.restore_context();
    }

    fn restore_context(&mut self) {
        let task = &mut self.tasks[self.current_task];
        if let Some(task) = task {
            unsafe {
                restore_context(task.sp, task.pc);
            }
        }
    }
}

struct Task {
    stack: [u32; TASK_STACK_SIZE],
    sp: u32,
    pc: u32,
    priority: u8,
    state: TaskState,
}

enum TaskState {
    Ready,
    Running,
    Blocked,
}

#[no_mangle]
fn save_context(sp: u32, pc: u32) {
    let task_manager = &raw mut TASK_MANAGER;

    unsafe {
        (*task_manager).save_context(sp, pc);
    }
}

extern "C" {
    fn restore_context(sp: u32, pc: u32);
}
