mod fs;
mod process;
mod time;

use config::syscall::*;

use crate::syscall::{fs::*, process::*, time::*};

/// handle syscall exception with `sycall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
	match syscall_id {
		WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
		EXIT => sys_exit(args[0] as i32),
		NANOSLEEP => sys_nanosleep(args[0] as *const KernelTimespec, args[1] as *mut KernelTimespec),
		YIELD => sys_yield(),
		GETTIMEOFDAY => sys_gettimeofday(args[0] as *mut TimeVal, args[1]),
		_ => panic!("Unsupported syscall_id: {}", syscall_id),
	}
}
