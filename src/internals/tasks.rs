use core::cell::UnsafeCell;

const STACK_SIZE: usize = 1024;
const MAX_TASKS: usize = 4;

#[derive(PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Terminated,
    Waiting,
    Stored,
}

pub struct TaskContext {
    pub sp: u32,
    pub pc: u32,
}

pub struct Task {
    id: usize,
    pub state: TaskState,
    pub context: TaskContext,
    stack: [u32; STACK_SIZE],
}

impl Task {
    const fn empty() -> Self {
        Task {
            id: 0,
            state: TaskState::Terminated,
            context: TaskContext { sp: 0, pc: 0 },
            stack: [0; STACK_SIZE],
        }
    }

    #[no_mangle]
    fn setup_stack(&mut self) {
        self.context.sp = (&self.stack[STACK_SIZE - 1] as *const u32) as u32;
    }

    fn executable(&self) -> bool {
        matches!(self.state, TaskState::Ready | TaskState::Stored)
    }
}

pub struct Scheduler {
    tasks: [UnsafeCell<Task>; MAX_TASKS],
    pub current_index: Option<usize>,
}

impl Scheduler {
    const fn new() -> Self {
        Scheduler {
            tasks: [const { UnsafeCell::new(Task::empty()) }; MAX_TASKS],
            current_index: None,
        }
    }

    fn init(&mut self) {
        for i in 0..MAX_TASKS {
            let task = self.task_mut(i);
            task.id = i;
        }
    }

    #[allow(dead_code)]
    fn task(&self, index: usize) -> &Task {
        unsafe { &*self.tasks[index].get() }
    }

    #[allow(clippy::mut_from_ref)]
    fn task_mut(&self, index: usize) -> &mut Task {
        unsafe { &mut *self.tasks[index].get() }
    }

    pub fn current(&mut self) -> Option<&mut Task> {
        self.current_index
            .map(move |index| self.task_mut(index))
            .filter(|task| task.state == TaskState::Running)
    }

    pub fn terminate(&mut self, index: usize) {
        let task = self.task_mut(index);
        task.state = TaskState::Terminated;
    }

    fn task_with_state(&self, state: TaskState) -> Option<&mut Task> {
        let initial_index = self.current_index.unwrap_or(0);
        let mut index = initial_index;

        loop {
            let current_task = self.task_mut(index);
            if current_task.state == state {
                return Some(current_task);
            }

            index = (index + 1) % MAX_TASKS;
            if index == initial_index {
                break;
            }
        }

        None
    }

    fn next_task(&mut self) -> Option<&mut Task> {
        let initial_index = self.current_index.unwrap_or(0);
        let mut index = initial_index;

        loop {
            let current_task = self.task_mut(index);
            if current_task.executable() {
                return Some(current_task);
            }

            index = (index + 1) % MAX_TASKS;
            if index == initial_index {
                break;
            }
        }

        None
    }

    pub fn create_task(&mut self, entry_point: fn()) -> Option<usize> {
        let task_id = match self.task_with_state(TaskState::Terminated) {
            Some(task) => task.id,
            None => return None,
        };

        let task = self.task_mut(task_id);
        task.state = TaskState::Ready;
        task.setup_stack();
        task.context.pc = entry_point as usize as u32;
        Some(task.id)
    }

    pub fn switch(&mut self) {
        let next_task_id = match self.next_task() {
            Some(task) => task.id,
            None => return,
        };

        self.current_index = Some(next_task_id);

        let task = self.task_mut(next_task_id);

        match task.state {
            TaskState::Ready => {
                task.state = TaskState::Running;
                unsafe {
                    switch_context(task.context.sp, task.context.pc);
                }
                task.state = TaskState::Terminated;
            }
            TaskState::Stored => {
                task.state = TaskState::Running;
                unsafe {
                    restore_context(task.context.sp, task.context.pc);
                }
                task.state = TaskState::Terminated;
            }
            _ => {}
        }
    }
}

static mut SCHEDULER: Scheduler = Scheduler::new();

#[allow(static_mut_refs)]
pub fn scheduler() -> &'static mut Scheduler {
    unsafe { &mut SCHEDULER }
}

pub fn init() {
    let scheduler = scheduler();
    scheduler.init();
}

pub fn create_task(entry_point: fn()) -> Option<usize> {
    let scheduler = scheduler();
    scheduler.create_task(entry_point)
}

extern "C" {
    fn switch_context(sp: u32, pc: u32);
    fn restore_context(sp: u32, px: u32);
}
