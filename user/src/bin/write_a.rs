#![no_std]
#![no_main]

use user::{info, print, println, syscall::sys_yield};

const WIDTH: usize = 10;
const HEIGHT: usize = 5;

#[unsafe(no_mangle)]
fn main() -> i32 {
	for i in 0..HEIGHT {
		for _ in 0..WIDTH {
			print!("A");
		}
        println!("");
		info!(" [{}/{}]", i + 1, HEIGHT);
		sys_yield();
	}
	info!("Test write_a OK!");
	0
}
