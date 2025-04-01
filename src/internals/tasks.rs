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
    /// Creates a new task instance with a terminated state and default values.
    ///
    /// The task is initialized with:
    /// - an ID of `0`
    /// - a state of `Terminated`
    /// - a context with both the stack pointer and program counter set to `0`
    /// - an empty page represented by `L2SmallPageTableEntry::empty()`
    ///
    /// # Examples
    ///
    /// ```
    /// let task = Task::empty();
    /// assert_eq!(task.state, TaskState::Terminated);
    /// assert_eq!(task.context.sp, 0);
    /// assert_eq!(task.context.pc, 0);
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

    /// Terminates the task by setting its state to `TaskState::Terminated` and unregistering its memory page.
    ///
    /// This method marks the task as no longer active. In addition to updating the task’s state to terminated,
    /// it performs necessary cleanup by unregistering the memory page associated with the task.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate_name::{Task, TaskState};
    /// let mut task = Task::empty();
    /// // Simulate an active task state
    /// task.state = TaskState::Running;
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

    /// Returns a mutable reference to the next executable task in the scheduler, or `None` if none is available.
    ///
    /// Starting from the current task index (or index 0 if not set), this method iterates through the task list in a cyclic manner.
    /// If it finds a task that is executable, it additionally checks if the task is in a waiting state and if its waiting period
    /// has elapsed (using `millis()`). If so, the task state is updated to `Stored` before being returned.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new scheduler instance and initialize tasks as needed.
    /// let mut scheduler = Scheduler::new();
    /// 
    /// // Attempt to retrieve the next executable task.
    /// if let Some(task) = scheduler.next_task() {
    ///     // A task was found and can be processed further.
    ///     task.state = TaskState::Running;
    /// } else {
    ///     // No executable task is available.
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
    /// This function searches for a task in the `Terminated` state and reinitializes it into a runnable task. It allocates a new memory page (using
    /// a `L2SmallPageTableEntry`) for the task, updates the task's state to `Ready`, sets the stack pointer to the end of the newly allocated page,
    /// and assigns the provided entry point as the program counter. If no terminated task is available or if page allocation fails, the creation
    /// process is aborted and `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `entry_point` - The function that will execute when the task is scheduled.
    ///
    /// # Returns
    ///
    /// Returns `Some(task_id)` if the task is created successfully, or `None` if creation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// fn example_task() {
    ///     // Task code goes here.
    /// }
    ///
    /// // Assume `scheduler` is a mutable instance of Scheduler.
    /// if let Some(task_id) = scheduler.create_task(example_task) {
    ///     println!("Task created with id: {}", task_id);
    /// } else {
    ///     println!("Failed to create task.");
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

    /// Switches to the next executable task in the scheduler.
    /// 
    /// This method searches for the next task that is eligible for execution. If a task in either the `Ready` or
    /// `Stored` state is found, the scheduler updates its current index, marks the task as running, registers its
    /// associated memory page, and then performs the appropriate context switch. If no executable task is available,
    /// the method exits without taking any action.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use your_crate::Scheduler;
    /// 
    /// let mut scheduler = Scheduler::new();
    /// // Assume tasks have been created and initialized appropriately.
    /// scheduler.switch();
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
