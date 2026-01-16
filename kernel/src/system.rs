use core::{fmt::{self, Write}, panic::PanicInfo};

use riscv::register::time;
use sbi_rt::{NoReason, Shutdown, SystemFailure, console_write_byte, system_reset};

use crate::error;

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::system::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::system::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

struct Stdout;

impl Write for Stdout {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for c in s.chars() {
			console_write_byte(c as u8).map_err(|_| fmt::Error)?;
		}
		Ok(())
	}
}

pub fn print(args: fmt::Arguments) { Stdout.write_fmt(args).unwrap(); }

/// We need to use `#[panic_handler]` to
/// specify [`panic_handler`] as panic handler.
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	let err = info.message();
	if let Some(location) = info.location() {
		error!("Panicked at {}:{} {}", location.file(), location.line(), err);
	} else {
		error!("Panicked: {}", err);
	}
	// If OS is panic, shutdown the computer.
	shutdown(true)
}

/// `failure` to represent whether the os is exit normally.
pub fn shutdown(failure: bool) -> ! {
	if failure {
		system_reset(Shutdown, SystemFailure);
	} else {
		system_reset(Shutdown, NoReason);
	}
	unreachable!()
}

/// Sleep for the specified number of milliseconds.
/// Uses the mtime register which runs at 10MHz.
pub fn sleep_ms(ms: u64) {
	// mtime frequency: 10MHz = 10,000,000 cycles per second
	// cycles = ms * 10,000,000 / 1000 = ms * 10,000
	let cycles = ms * 10_000;
	sleep_cycles(cycles);
}

/// Sleep for the specified number of microseconds.
/// Uses the mtime register which runs at 10MHz.
pub fn sleep_us(us: u64) {
	// mtime frequency: 10MHz = 10,000,000 cycles per second
	// cycles = us * 10,000,000 / 1,000,000 = us * 10
	let cycles = us * 10;
	sleep_cycles(cycles);
}

fn sleep_cycles(cycles: u64) {
	let start = time::read();
	let target = start.wrapping_add(cycles as usize);

	// Handle wrapping case
	if target < start {
		while time::read() >= start {}
	}
	while time::read() < target {}
}
