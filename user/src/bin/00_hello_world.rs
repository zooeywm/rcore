//! Hello world application

#![no_std]
#![no_main]

use user::info;

#[unsafe(no_mangle)]
fn main() -> i32 {
	info!("Hello, world!");
	0
}
