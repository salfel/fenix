use core::{cell::UnsafeCell, ptr};

use shared::alloc::heap::BumpAllocator;

use super::mmu::L2SmallPageTableEntry;
use crate::sysclock::millis;

const MAX_TASKS: usize = 4;
const STACK_GUARD: usize = 1024;

const CODE_PAGE_LOCATION: u32 = 0x0;
const DATA_PAGE_LOCATION: u32 = 0x1000;

#[derive(PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Terminated,
    Waiting { until: u32 },
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
    pub allocator: BumpAllocator,
    code_page: L2SmallPageTableEntry,
    data_page: L2SmallPageTableEntry,
}

impl Task {
    const fn empty() -> Self {
        Task {
            id: 0,
            state: TaskState::Terminated,
            context: TaskContext { sp: 0, pc: 0 },
            allocator: BumpAllocator::new(),
            code_page: L2SmallPageTableEntry::empty(),
            data_page: L2SmallPageTableEntry::empty(),
        }
    }

    fn executable(&mut self) -> bool {
        match self.state {
            TaskState::Ready | TaskState::Stored => true,
            TaskState::Waiting { until } => {
                if millis() >= until {
                    self.state = TaskState::Stored;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn terminate(&mut self) {
        self.state = TaskState::Terminated;
        self.data_page.unregister();
        self.code_page.unregister();
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
    pub fn task_mut(&self, index: usize) -> &mut Task {
        unsafe { &mut *self.tasks[index].get() }
    }

    pub fn current(&mut self) -> Option<&mut Task> {
        self.current_index
            .map(move |index| self.task_mut(index))
            .filter(|task| task.state == TaskState::Running)
    }

    pub fn cycle(&mut self) {
        if let Some(ref mut index) = self.current_index {
            *index = (*index + 1) % MAX_TASKS;
        }
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

    pub fn create_task(&mut self, code: &[u8]) -> Option<usize> {
        let task_id = self.task_with_state(TaskState::Terminated)?.id;

        let code_page = L2SmallPageTableEntry::try_new(CODE_PAGE_LOCATION, Some(task_id as u32))?;
        let data_page = L2SmallPageTableEntry::try_new(DATA_PAGE_LOCATION, Some(task_id as u32))?;

        let dest = code_page.start() as *mut u8;
        unsafe {
            ptr::copy_nonoverlapping(code.as_ptr(), dest, code.len());
        }

        let task = self.task_mut(task_id);
        task.code_page = code_page;
        task.data_page = data_page;
        task.state = TaskState::Ready;
        task.context.sp = task.data_page.end();
        task.context.pc = task.code_page.start();
        task.allocator
            .init(task.data_page.start() as usize, task.data_page.end() as usize - STACK_GUARD);
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
                task.code_page.register();
                task.data_page.register();
                unsafe {
                    switch_context(task.context.sp, task.context.pc);
                }
            }
            TaskState::Stored => {
                task.state = TaskState::Running;
                task.code_page.register();
                task.data_page.register();
                unsafe {
                    restore_context(task.context.sp, task.context.pc);
                }
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

pub fn create_task(code: &[u8]) -> Option<usize> {
    let scheduler = scheduler();
    scheduler.create_task(code)
}

extern "C" {
    fn switch_context(sp: u32, pc: u32) -> !;
    fn restore_context(sp: u32, pc: u32) -> !;
}
