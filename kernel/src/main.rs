#![no_std]
#![no_main]
#![feature(step_trait)]
// #![feature(alloc_error_handler)]

// extern crate alloc;
use core::{arch::global_asm, error};

#[macro_use]
mod console;

mod boards;
mod config;
mod lang_items;
mod loader;
mod log;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod task;
mod trap;

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/link_app.S"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
	unsafe extern "C" {
		safe fn stext(); // begin addr of text segment
		safe fn etext(); // end addr of text segment
		safe fn srodata(); // start addr of Read-Only data segment
		safe fn erodata(); // end addr of Read-Only data ssegment
		safe fn sdata(); // start addr of data segment
		safe fn edata(); // end addr of data segment
		safe fn sbss(); // start addr of BSS segment
		safe fn ebss(); // end addr of BSS segment
		safe fn boot_stack_lower_bound(); // stack lower bound
		safe fn boot_stack_top(); // stack top
	}
	(sbss as *const () as usize..ebss as *const () as usize)
		.for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });

	trace!("rcore started!");
	trace!("text [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
	trace!(".rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
	trace!(".data [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
	trace!(
		"boot_stack top=bottom={:#x}, lower_bound={:#x}",
		boot_stack_top as *const () as usize, boot_stack_lower_bound as *const () as usize
	);
	trace!(".bss [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);

	trap::init();
	loader::load_apps();
	trap::enable_timer_interrupt();
	sbi::set_next_trigger();
	task::run_first_task();
	panic!("Unreachable in rust_main!");
}
