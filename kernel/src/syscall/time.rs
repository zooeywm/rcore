use config::{errno::EINVAL, syscall::KernelTimespec};

use crate::sbi::sleep_ns;

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
