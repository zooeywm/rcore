use core::arch::asm;

use crate::trace;

pub unsafe fn print_stack_trace() {
	let mut fp: *const usize;
	unsafe {
		asm!("mv {}, fp", out(reg) fp);
	}

	trace!("== Begin stack trace ==");
	while !fp.is_null() {
		let saved_ra = unsafe { *fp.sub(1) };
		let saved_fp = unsafe { *fp.sub(2) };

		trace!("0x{:016x}, fp = 0x{:016x}", saved_ra, saved_fp);

		fp = saved_fp as *const usize;
	}
	trace!("== End stack trace ==");
}
