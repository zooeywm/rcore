mod global {
	use core::cell::{RefCell, RefMut};

	use lazy_static::lazy_static;

	use crate::{batch::AppManager, config::MAX_APP_NUM};

	/// Wrap a static data structure inside it so that we are able to access it
	/// without any `unsafe`.
	///
	/// We should only use it in Uniprocessor.
	///
	/// In order to get mutable reference of inner data, call
	/// [`UpSafeCell::exclusive_access()`]
	pub struct UPSafeCell<T> {
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
        pub(super) static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
            UPSafeCell::new({
                unsafe extern "C" {
                    fn _num_app();
                }
                // Get app_num array start ptr
                let num_app_ptr = _num_app as *const () as *const usize;
                // Read app_num array start address
                let num_app = num_app_ptr.read_volatile();
                // Init an array of each app's start address and the last app's end address.
                let mut app_start = [0; MAX_APP_NUM + 1];
                // Add 1 to num_app_ptr to start from the first app's address, read raw apps' start.
                let app_start_raw = core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
                // Apply these addresses to app_start array.
                app_start[..=num_app].copy_from_slice(app_start_raw);
                AppManager{
                    num_app,
                    current_app: 0,
                    app_start
                }
            })
        };
	}
}

use core::{arch::asm, slice::{from_raw_parts, from_raw_parts_mut}};

use global::APP_MANAGER;

use crate::{config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE}, debug, info, system::shutdown, trap::context::TrapContext};

/// Batch OS app manager.
struct AppManager {
	/// Num of apps.
	num_app:     usize,
	/// Current running app.
	current_app: usize,
	/// Each app's start address and last app's end address array.
	app_start:   [usize; MAX_APP_NUM + 1],
}

impl AppManager {
	pub fn log_app_info(&self) {
		debug!("num_app = {}", self.num_app);
		for i in 0..self.num_app {
			debug!("app_{i} [{:#x}, {:#x}]", self.app_start[i], self.app_start[i + 1]);
		}
	}

	pub fn get_current_app(&self) -> usize { self.current_app }

	pub fn move_to_next_app(&mut self) { self.current_app += 1; }

	unsafe fn load_app(&self, app_id: usize) {
		if app_id >= self.num_app {
			info!("All applications completed!");
			shutdown(false);
		}
		info!("Loading app_{}", app_id);
		// Clear app area
		unsafe {
			from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);

			let app_src = from_raw_parts(
				self.app_start[app_id] as *const u8,
				self.app_start[app_id + 1] - self.app_start[app_id],
			);
			// Load new app to app dest.
			from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len()).copy_from_slice(app_src);
			// It is really useless to clear the icache before executing the first
			// application, because there is no relevant content in the icache at that time.
			// However, when executing subsequent applications, the instructions of the
			// previous application will be cached in the icache, so it needs to be cleared
			// manually.
			asm!("fence.i")
		}
	}
}

#[repr(align(4096))]
struct KernelStack {
	data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
	data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };

static USER_STACK: UserStack = UserStack { data: [0; USER_STACK_SIZE] };

impl KernelStack {
	/// Because Stack in RISC-V is downward grossing, we calculate with data ptr +
	/// Stack size
	fn get_sp(&self) -> usize { KERNEL_STACK_SIZE + self.data.as_ptr() as usize }

	fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
		let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
		unsafe {
			*cx_ptr = cx;
			cx_ptr.as_mut().unwrap()
		}
	}
}

impl UserStack {
	fn get_sp(&self) -> usize { USER_STACK_SIZE + self.data.as_ptr() as usize }
}

pub fn init() { APP_MANAGER.exclusive_access().log_app_info(); }

/// Run next app
pub fn run_next_app() -> ! {
	let mut app_manager = APP_MANAGER.exclusive_access();
	let current_app = app_manager.get_current_app();
	unsafe {
		app_manager.load_app(current_app);
	}
	app_manager.move_to_next_app();
	drop(app_manager);
	unsafe extern "C" {
		fn __restore(cx_addr: usize);
	}
	unsafe {
		__restore(
			KERNEL_STACK.push_context(TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp()))
				as *const _ as usize,
		);
	}
	unreachable!("restore will call ret, thus this will never reach");
}
