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

	println!("Hello, user!");
	trace!("text [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
	debug!(".rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
	info!(".data [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
	warn!(".bss [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);
	error!("This is an error log");
	info!("Sleep 500ms");
	// sleep_ms(500);
	info!("Sleep 100000us(100ms)");
	// sleep_us(100000);
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
