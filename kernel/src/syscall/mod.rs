mod fs;
mod process;
mod time;

use config::syscall::*;

use crate::syscall::{fs::sys_write, process::{sys_exit, sys_yield}, time::sys_nanosleep};

/// handle syscall exception with `sycall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
	match syscall_id {
		WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
		EXIT => sys_exit(args[0] as i32),
		NANOSLEEP => sys_nanosleep(args[0] as *const KernelTimespec, args[1] as *mut KernelTimespec),
		SYSCALL_YIELD => sys_yield(),
		_ => panic!("Unsupported syscall_id: {}", syscall_id),
	}
}
