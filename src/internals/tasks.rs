use core::cell::UnsafeCell;

const STACK_SIZE: usize = 256;
const MAX_TASKS: usize = 4;

#[derive(PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Terminated,
    Waiting,
}

struct TaskContext {
    sp: u32,
    pc: u32,
}

pub struct Task {
    id: usize,
    pub state: TaskState,
    context: TaskContext,
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

    fn setup_stack(&mut self) {
        self.context.sp = self.stack[STACK_SIZE - 1] as *const u32 as u32;
    }
}

pub struct Scheduler {
    tasks: [UnsafeCell<Task>; MAX_TASKS],
    current_index: Option<usize>,
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

    fn task_with_state(&mut self, state: TaskState) -> Option<&mut Task> {
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

    pub fn create_task(&mut self, entry_point: fn()) -> Option<usize> {
        let task = self.task_with_state(TaskState::Terminated);

        match task {
            Some(task) => {
                task.state = TaskState::Ready;
                task.setup_stack();
                task.context.pc = entry_point as usize as u32;
                Some(task.id)
            }
            None => None,
        }
    }

    pub fn switch(&mut self) {
        if let Some(task) = self.task_with_state(TaskState::Ready) {
            unsafe {
                switch_context(task.context.sp, task.context.pc);
            }
            task.state = TaskState::Terminated;
        }
    }
}

#[link_section = "user_stack_start"]
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
}
