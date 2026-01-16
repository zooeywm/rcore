use core::{fmt::{Arguments, Write}, panic::PanicInfo};

use crate::{config::STDOUT, error, syscall::sys_write};

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::system::print(format_args!($fmt $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::system::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

struct Stdout;

impl Write for Stdout {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		sys_write(STDOUT, s.as_bytes());
		Ok(())
	}
}

pub fn print(args: Arguments) { Stdout.write_fmt(args).unwrap() }

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
	// When application panic, pend and wait for os instruction.
	loop {}
}
