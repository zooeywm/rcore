use config::{errno::*, fd::*, syscall::*};

use crate::{batch::run_next_app, print, system::sleep_ns, trace};

/// handle syscall exception with `sycall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
	match syscall_id {
		WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
		EXIT => sys_exit(args[0] as i32),
		NANOSLEEP => sys_nanosleep(args[0] as *const KernelTimespec, args[1] as *mut KernelTimespec),
		_ => panic!("Unsupported syscall_id: {}", syscall_id),
	}
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
	trace!("Application exited with code {}", exit_code);
	run_next_app()
}

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
	match fd {
		STDOUT => {
			let slice = unsafe { core::slice::from_raw_parts(buf, len) };
			let str = core::str::from_utf8(slice).expect("sys_write not utf8 string");
			print!("{}", str);
			len as isize
		}
		_ => {
			panic!("Unsupported fd in sys_write!");
		}
	}
}

/// Implementation of `sys_nanosleep`.
///
/// # Arguments
/// * `req` - Pointer to the requested sleep time. Must not be null.
/// * `rem` - Optional pointer to store remaining time if the sleep is
///   interrupted.
///
/// # Notes
/// This is a kernel-space implementation and does not call the Linux syscall.
pub fn sys_nanosleep(req: *const KernelTimespec, rem: *mut KernelTimespec) -> isize {
	if req.is_null() {
		return -EINVAL;
	}

	// Safely read the user-provided timespec
	let ts = unsafe { &*req }; // stack-local copy

	// Convert seconds and nanoseconds into total nanoseconds
	let total_ns = ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64;

	// Perform the sleep using busy-wait
	sleep_ns(total_ns);

	// Handle the remaining time if interrupted (not implemented yet)
	if !rem.is_null() {
		unimplemented!("Remaining time handling requires signal/interruption logic");
	}

	0
}
