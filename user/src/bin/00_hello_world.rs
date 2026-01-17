//! Hello world application

#![no_std]
#![no_main]

use core::ptr;

use config::syscall::KernelTimespec;
use user::{info, syscall::sys_nanosleep};

#[unsafe(no_mangle)]
fn main() -> i32 {
	info!("Hello, world!");
	info!("Sleep 1s");
	sys_nanosleep(&KernelTimespec::sec(1), ptr::null_mut());
	info!("Sleep finished!");
	info!("Sleep 1s Again");
	sys_nanosleep(&KernelTimespec::sec(1), ptr::null_mut());
	info!("Sleep again finished!");
	0
}
