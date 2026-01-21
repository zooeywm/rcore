use config::syscall::TimeVal;

use crate::{config::MICRO_PER_SEC, sbi::get_time_us, task::{exit_current_and_run_next, suspend_current_and_run_next}, trace};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
	trace!("Application exited with code {}", exit_code);
	exit_current_and_run_next();
	panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
	suspend_current_and_run_next();
	0
}

pub fn sys_gettimeofday(ts: *mut TimeVal, _tz: usize) -> isize {
	let us = get_time_us();
	unsafe {
		*ts = TimeVal { sec: us / MICRO_PER_SEC, usec: us % MICRO_PER_SEC };
	}
	0
}
