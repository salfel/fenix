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
    /// Constructs an empty `Task` with default inactive values.
    ///
    /// The returned task has an `id` of 0, is marked as `Terminated`,
    /// and its execution context is set with both the stack pointer and program counter at 0.
    /// Additionally, the task's memory page is initialized to an empty state using `L2SmallPageTableEntry::empty()`.
    ///
    /// # Examples
    ///
    /// ```
    /// let empty_task = Task::empty();
    /// assert_eq!(empty_task.id, 0);
    /// assert_eq!(empty_task.state, TaskState::Terminated);
    /// assert_eq!(empty_task.context.sp, 0);
    /// assert_eq!(empty_task.context.pc, 0);
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

    /// Terminates the task by marking it as inactive and unregistering its associated memory page.
    /// 
    /// The task's state is set to `Terminated`, ensuring it is no longer scheduled for execution.
    /// Additionally, the task's memory page is unregistered to release any resources tied to it.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use crate::{Task, TaskState};
    /// 
    /// let mut task = Task::empty();
    /// task.terminate();
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

    /// Searches for and returns the next executable task in a circular manner.
    ///
    /// Starting from the current task index (or 0 if unset), this method iterates over all tasks.
    /// If it encounters a task that is executable, it checks whether the task is in a `Waiting` state
    /// and if its designated resume time (as determined by `millis()`) has passed. In that case, the
    /// task's state is updated to `Stored` before being returned. If no executable task is found during
    /// a full cycle, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// // Initialize the scheduler and its tasks.
    /// let mut scheduler = Scheduler::new();
    /// scheduler.init();
    ///
    /// // Retrieve the next executable task, if any.
    /// if let Some(task) = scheduler.next_task() {
    ///     // `task` is ready to run or has just been updated from Waiting to Stored.
    ///     // Process the executable task as needed.
    /// } else {
    ///     // No executable task was found in the scheduler.
    /// }
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

    /// Creates a new task with the specified entry point.
    ///
    /// This function reuses a terminated task slot to create a new task. It allocates a memory page
    /// for the task, sets the task's state to ready, and initializes its context by setting the stack
    /// pointer to the end of the allocated page and the program counter to the provided entry point.
    /// Returns the task ID on success; returns `None` if no terminated task slot is available or if page
    /// allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `scheduler` is a mutable instance of `Scheduler`, properly initialized with task slots.
    /// fn entry_point() {
    ///     // Task code goes here.
    /// }
    ///
    /// if let Some(task_id) = scheduler.create_task(entry_point) {
    ///     println!("Created task with ID: {}", task_id);
    /// } else {
    ///     eprintln!("Task creation failed");
    /// }
    /// ```
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
    /// This function looks for the next task that is ready or stored by invoking `next_task()`.
    /// If an eligible task is found, it updates the scheduler's current task index and transitions
    /// the task's state to `Running`. For tasks in the `Ready` state, it registers the task's memory
    /// page and calls the external `switch_context` function to switch execution. For tasks in the
    /// `Stored` state, it similarly registers the page and calls `restore_context`.
    /// If no suitable task is found, the function exits without making any context changes.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Example usage of the scheduler's `switch` function:
    /// let mut scheduler = Scheduler::new();
    /// scheduler.init();
    ///
    /// // Create a new task with a dummy entry point for demonstration purposes.
    /// let task_id = scheduler.create_task(dummy_task_entry).unwrap();
    ///
    /// // Set the task's state to Ready to make it executable.
    /// {
    ///     let task = scheduler.task_mut(task_id);
    ///     task.state = TaskState::Ready;
    /// }
    ///
    /// // Switch to the next executable task.
    /// scheduler.switch();
    ///
    /// fn dummy_task_entry() {
    ///     // Task execution code goes here.
    /// }
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
