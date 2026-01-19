#![feature(linkage)]
#![no_std]

use crate::syscall::sys_exit;

mod log;
mod stack_trace;
pub mod syscall;
pub mod system;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
	unsafe extern "C" {
		safe fn stext(); // begin addr of text segment
		safe fn etext(); // end addr of text segment
		safe fn srodata(); // start addr of Read-Only data segment
		safe fn erodata(); // end addr of Read-Only data ssegment
		safe fn sdata(); // start addr of data segment
		safe fn edata(); // end addr of data segment
		safe fn sbss(); // start addr of BSS segment
		safe fn ebss(); // end addr of BSS segment
	}
	(sbss as *const () as usize..ebss as *const () as usize)
		.for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });

	trace!("user app loaded");
	trace!("text [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
	trace!(".rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
	trace!(".data [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
	trace!(".bss [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);
	trace!("This is an error log");
	sys_exit(main());
	unreachable!("unreachable after sys_exit!");
}

/// Weak linkage, to make it pass compile when bin lack of main function.
/// But will panic at runtime.
#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
	panic!("Cannot find main!");
}
