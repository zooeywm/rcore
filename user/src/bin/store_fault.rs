//! Test store fault.

#![no_std]
#![no_main]

use user::warn;

#[unsafe(no_mangle)]
fn main() -> i32 {
	warn!(
		"Into Test store_fault, we will insert an invalid store operation, kernel should kill this application!"
	);
	unsafe {
		core::ptr::null_mut::<u8>().write_volatile(0);
	}
	0
}
