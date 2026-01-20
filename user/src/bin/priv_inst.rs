//! Test access S-CSR in U mode.

#![no_std]
#![no_main]

use riscv::register::sstatus::{self, SPP};
use user::warn;

#[unsafe(no_mangle)]
fn main() -> i32 {
	warn!("Try to access privileged CSR in U mode, kernel should kill this application!");
	unsafe { sstatus::set_spp(SPP::User) }
	0
}
