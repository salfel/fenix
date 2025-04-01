use core::cell::UnsafeCell;

use super::{mmu::L2SmallPageTableEntry, sysclock::millis};

pub const MAX_TASKS: usize = 4;

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
    page: L2SmallPageTableEntry,
}

impl Task {
    /// Creates a new terminated task with default settings.
    ///
    /// The returned task has an ID of 0, a state of `TaskState::Terminated`, a default context
    /// (with both the stack pointer and program counter set to 0), and an empty memory page
    /// provided by `L2SmallPageTableEntry::empty()`. Being a `const fn`, it can be evaluated
    /// at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// let task = Task::empty();
    /// assert_eq!(task.id, 0);
    /// assert_eq!(task.state, TaskState::Terminated);
    /// assert_eq!(task.context.sp, 0);
    /// assert_eq!(task.context.pc, 0);
    /// // Check that the page entry is empty according to the L2SmallPageTableEntry API:
    /// assert!(task.page.is_empty());
    /// ```
    const fn empty() -> Self {
        Task {
            id: 0,
            state: TaskState::Terminated,
            context: TaskContext { sp: 0, pc: 0 },
            page: L2SmallPageTableEntry::empty(),
        }
    }

    fn executable(&self) -> bool {
        matches!(
            self.state,
            TaskState::Ready | TaskState::Stored | TaskState::Waiting { .. }
        )
    }

    /// Terminates the task by setting its state to `Terminated` and unregistering its memory page.
    ///
    /// This method marks the task as terminated and cleans up its associated memory resources by
    /// unregistering the allocated page.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{Task, TaskState, L2SmallPageTableEntry};
    /// // Create a task in a running state for demonstration purposes.
    /// let mut task = Task {
    ///     state: TaskState::Running,
    ///     page: L2SmallPageTableEntry::empty(), // Replace with the appropriate constructor if needed
    ///     // Other necessary fields can be added here.
    /// };
    ///
    /// // Terminate the task.
    /// task.terminate();
    ///
    /// // Verify that the task's state is now terminated.
    /// assert_eq!(task.state, TaskState::Terminated);
    /// ```
    pub fn terminate(&mut self) {
        self.state = TaskState::Terminated;
        self.page.unregister();
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

    /// Returns the next task that is ready for execution.
    ///
    /// This method iterates over the scheduler's tasks starting from the current task index in a circular manner.
    /// When a task is found in a waiting state, if the current time (from `millis()`) has met or surpassed its wait deadline,
    /// the task's state is updated to stored, making it eligible for execution. The function returns a mutable reference
    /// to the first executable task found, or `None` if no executable task exists after a full cycle.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::scheduler::{Scheduler, TaskState, MAX_TASKS};
    /// let mut scheduler = Scheduler::new();
    /// scheduler.current_index = Some(0);
    /// // Set the first task as stored (executable).
    /// scheduler.tasks[0].state = TaskState::Stored;
    ///
    /// let task = scheduler.next_task();
    /// assert!(task.is_some());
    /// ```
    fn next_task(&mut self) -> Option<&mut Task> {
        let initial_index = self.current_index.unwrap_or(0);
        let mut index = initial_index;

        loop {
            let current_task = self.task_mut(index);
            if current_task.executable() {
                if let TaskState::Waiting { until } = current_task.state {
                    if millis() >= until {
                        current_task.state = TaskState::Stored;
                    }
                }

                return Some(current_task);
            }

            index = (index + 1) % MAX_TASKS;
            if index == initial_index {
                break;
            }
        }

        None
    }

    //// Define a simple task entry function.
    pub fn create_task(&mut self, entry_point: fn()) -> Option<usize> {
        let task_id = self.task_with_state(TaskState::Terminated)?.id;

        let page = L2SmallPageTableEntry::try_new(Some(task_id as u32))?;

        let task = self.task_mut(task_id);
        task.page = page;
        task.state = TaskState::Ready;
        task.context.sp = task.page.end();
        task.context.pc = entry_point as usize as u32;
        Some(task.id)
    }

    /// Switches execution to the next executable task.
    ///
    /// This method searches for the next task that is ready for execution by calling `next_task()`. If a task is found,
    /// it sets the scheduler's current index to that task's ID and transitions the task's state to `Running`. Depending on
    /// the task's initial state, it then either initiates a context switch via `switch_context` (if the task was `Ready`)
    /// or restores its previous context via `restore_context` (if the task was `Stored`). If no executable task is available,
    /// the method returns without making changes.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new scheduler and initialize it.
    /// let mut scheduler = Scheduler::new();
    /// scheduler.init();
    ///
    /// // Manually set up a dummy task at index 0 as ready with sample context values.
    /// {
    ///     let task = scheduler.task_mut(0);
    ///     task.state = TaskState::Ready;
    ///     task.context = TaskContext { sp: 1000, pc: 2000 };
    ///     // Simulate page setup if necessary; in practice, proper page initialization would be required.
    /// }
    ///
    /// // Attempt to switch to the next executable task.
    /// // In this simple setup, task 0 should be selected and its state updated to Running.
    /// scheduler.switch();
    ///
    /// // Verify that the task's state has been updated to Running.
    /// let task0 = scheduler.task(0);
    /// assert_eq!(task0.state, TaskState::Running);
    /// ```
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
                task.page.register();
                unsafe {
                    switch_context(task.context.sp, task.context.pc);
                }
            }
            TaskState::Stored => {
                task.state = TaskState::Running;
                task.page.register();
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

pub fn create_task(entry_point: fn()) -> Option<usize> {
    let scheduler = scheduler();
    scheduler.create_task(entry_point)
}

extern "C" {
    fn switch_context(sp: u32, pc: u32);
    fn restore_context(sp: u32, pc: u32);
}
