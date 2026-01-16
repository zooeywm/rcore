#![no_std]
#![no_main]

use core::arch::global_asm;

use crate::system::{sleep_ms, sleep_us};

mod config;
mod logger;
mod system;

global_asm!(include_str!("asm/entry.asm"));

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

	println!("[kernel] Hello, world!");
	trace!("[kernel] .text [{:#x}, {:#x})", stext as *const () as usize, etext as *const () as usize);
	debug!("[kernel] .rodata [{:#x}, {:#x})", srodata as *const () as usize, erodata as *const () as usize);
	info!("[kernel] .data [{:#x}, {:#x})", sdata as *const () as usize, edata as *const () as usize);
	warn!(
		"[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
		boot_stack_top as *const () as usize, boot_stack_lower_bound as *const () as usize
	);
	error!("[kernel] .bss [{:#x}, {:#x})", sbss as *const () as usize, ebss as *const () as usize);
	info!("Sleep 500ms");
	sleep_ms(500);
	info!("Sleep 100000us(100ms)");
	sleep_us(100000);
	panic!("Shutdown machine!");
}
