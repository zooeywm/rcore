use riscv::register::time;
use sbi_rt::{NoReason, Shutdown, SystemFailure, set_timer, system_reset};

use crate::config::{MICRO_PER_SEC, MTIME_FREQUENCY_HZ, TICKS_PER_SEC};

/// `failure` to represent whether the os is exit normally.
pub fn shutdown(failure: bool) -> ! {
	if failure {
		system_reset(Shutdown, SystemFailure);
	} else {
		system_reset(Shutdown, NoReason);
	}
	unreachable!()
}

/// Sleep for the specified number of nanoseconds
/// Uses the mtime register (10MHz tick → 100ns per tick)
pub fn sleep_ns(ns: u64) {
	// one tick = 100 ns
	let cycles = ns.div_ceil(100);
	sleep_ticks(cycles);
}

fn sleep_ticks(ticks: u64) {
	let start = time::read();
	let target = start.wrapping_add(ticks as usize);

	// Handle wrapping case
	if target < start {
		while time::read() >= start {}
	}
	while time::read() < target {}
}

pub fn set_next_trigger() {
	set_timer(time::read() as u64 + MTIME_FREQUENCY_HZ / TICKS_PER_SEC).expect("set_timer error");
}

pub fn get_time_us() -> u64 { time::read() as u64 / (MTIME_FREQUENCY_HZ / MICRO_PER_SEC) }
