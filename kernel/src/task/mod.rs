use crate::{config::MAX_APP_NUM, println, system::shutdown, task::{context::TaskContext, global::UPSafeCell, switch::__switch}};

mod context;
mod switch;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
	pub task_status: TaskStatus,
	pub task_cx:     TaskContext,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
	UnInit,
	Ready,
	Running,
	Exited,
}

mod global {
	use core::cell::{RefCell, RefMut};

	use lazy_static::lazy_static;

	use crate::{config::MAX_APP_NUM, loader::{get_num_app, init_app_cx}, task::{TaskControlBlock, TaskManager, TaskManagerInner, TaskStatus, context::TaskContext}};

	/// Wrap a static data structure inside it so that we are able to access it
	/// without any `unsafe`.
	///
	/// We should only use it in Uniprocessor.
	///
	/// In order to get mutable reference of inner data, call
	/// [`UpSafeCell::exclusive_access()`]
	pub(super) struct UPSafeCell<T> {
		inner: RefCell<T>,
	}

	unsafe impl<T> Sync for UPSafeCell<T> {}

	impl<T> UPSafeCell<T> {
		/// User is responsible to guarantee that inner struct is only used in
		/// Uniprocessor.
		unsafe fn new(value: T) -> Self { Self { inner: RefCell::new(value) } }

		/// Exclusive access inner data in [`UpSafeCell`]. Panic if the data has
		/// been borrowed.
		pub fn exclusive_access(&self) -> RefMut<'_, T> { self.inner.borrow_mut() }
	}

	// `lazy_static!` help us initialize a global variable at the first time it is
	// used.
	#[rustfmt::skip]
	lazy_static! {
        pub(super) static ref TASK_MANAGER: TaskManager = {
            let num_app = get_num_app();
            let mut tasks = [TaskControlBlock {
                task_cx: TaskContext::zero_init(),
                task_status: TaskStatus::UnInit,
            }; MAX_APP_NUM];
            for (i, task) in tasks.iter_mut().enumerate() {
                task.task_cx = TaskContext::goto_restore(init_app_cx(i));
                task.task_status = TaskStatus::Ready;
            }
            TaskManager {
                num_app,
                inner: unsafe {
                    UPSafeCell::new(TaskManagerInner {
                        tasks,
                        current_task: 0,
                    })
                },
            }
        };
	}
}

use global::TASK_MANAGER;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
	/// total number of tasks
	num_app: usize,
	/// use inner value to get mutable access
	inner:   UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
	/// task list
	tasks:        [TaskControlBlock; MAX_APP_NUM],
	/// id of current `Running` task
	current_task: usize,
}

impl TaskManager {
	/// Run the first task in task list.
	///
	/// Generally, the first task in task list is an idle task (we call it zero
	/// process later). But in ch3, we load apps statically, so the first task is
	/// a real app.
	fn run_first_task(&self) -> ! {
		let mut inner = self.inner.exclusive_access();
		let task0 = &mut inner.tasks[0];
		task0.task_status = TaskStatus::Running;
		let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
		drop(inner);
		let mut _unused = TaskContext::zero_init();
		// before this, we should drop local variables that must be dropped manually
		unsafe {
			__switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
		}
		panic!("unreachable in run_first_task!");
	}

	/// Change the status of current `Running` task into `Ready`.
	fn mark_current_suspended(&self) {
		let mut inner = self.inner.exclusive_access();
		let current = inner.current_task;
		inner.tasks[current].task_status = TaskStatus::Ready;
	}

	/// Change the status of current `Running` task into `Exited`.
	fn mark_current_exited(&self) {
		let mut inner = self.inner.exclusive_access();
		let current = inner.current_task;
		inner.tasks[current].task_status = TaskStatus::Exited;
	}

	/// Switch current `Running` task to the task we have found,
	/// or there is no `Ready` task and we can exit with all applications
	/// completed
	fn run_next_task(&self) {
		if let Some(next) = self.find_next_task() {
			let (current_task_cx_ptr, next_task_cx_ptr) = {
				let mut inner = self.inner.exclusive_access();
				let current = inner.current_task;
				inner.tasks[next].task_status = TaskStatus::Running;
				inner.current_task = next;
				(
					&mut inner.tasks[current].task_cx as *mut TaskContext,
					&inner.tasks[next].task_cx as *const TaskContext,
				)
			};
			// before this, we should drop local variables that must be dropped manually
			unsafe {
				__switch(current_task_cx_ptr, next_task_cx_ptr);
			}
			// go back to user mode
		} else {
			println!("All applications completed!");
			shutdown(false);
		}
	}

	/// Find next task to run and return app id.
	///
	/// In this case, we only return the first `Ready` task in task list.
	fn find_next_task(&self) -> Option<usize> {
		let inner = self.inner.exclusive_access();
		let current = inner.current_task;
		(current + 1..current + self.num_app + 1)
			.map(|id| id % self.num_app)
			.find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
	}
}

/// run first task
pub fn run_first_task() { TASK_MANAGER.run_first_task(); }

/// rust next task
fn run_next_task() { TASK_MANAGER.run_next_task(); }

/// suspend current task
fn mark_current_suspended() { TASK_MANAGER.mark_current_suspended(); }

/// exit current task
fn mark_current_exited() { TASK_MANAGER.mark_current_exited(); }

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
	mark_current_suspended();
	run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
	mark_current_exited();
	run_next_task();
}
