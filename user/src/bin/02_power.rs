//! Test power, array and mod

#![no_std]
#![no_main]

use user::info;

const SIZE: usize = 10;
const P: u32 = 3;
const STEP: usize = 100000;
const MOD: u32 = 10007;

#[unsafe(no_mangle)]
fn main() -> i32 {
	let mut pow = [0u32; SIZE];
	let mut index = 0;
	pow[index] = 1;
	for i in 1..=STEP {
		let last = pow[index];
		index = (index + 1) % SIZE;
		pow[index] = last * P % MOD;
		if i % 10000 == 0 {
			info!("({}^{})%{}={}", P, i, MOD, pow[index]);
		}
	}
	info!("Test power OK!");
	0
}
