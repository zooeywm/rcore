use core::{fmt::{self, Write}, panic::PanicInfo};

use sbi_rt::{NoReason, Shutdown, SystemFailure, console_write_byte, system_reset};

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
		println!("Panicked at {}:{} {}", location.file(), location.line(), err);
	} else {
		println!("Panicked: {}", err);
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
