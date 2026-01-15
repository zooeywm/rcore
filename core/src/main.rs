#![no_std]
#![no_main]

use core::arch::global_asm;

mod system;

global_asm!(include_str!("asm/entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
	clear_bss();
	println!("Hello, world!");
	panic!("Shutdown machine!");
}

fn clear_bss() {
	unsafe extern "C" {
		fn sbss();
		fn ebss();
	}
	(sbss as *const () as usize..ebss as *const () as usize)
		.for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
