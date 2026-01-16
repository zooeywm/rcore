//! Test call S-instruction in U mode.

#![no_std]
#![no_main]

use core::arch::asm;

use user::warn;

#[unsafe(no_mangle)]
fn main() -> i32 {
	warn!("Try to execute privileged instruction in U mode, kernel should kill this application!");
	unsafe {
		asm!("sret");
	}
	0
}
