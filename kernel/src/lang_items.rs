use core::panic::PanicInfo;

use crate::{error, sbi::shutdown, stack_trace::print_stack_trace};

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
	unsafe {
		print_stack_trace();
	}
	// If OS is panic, shutdown the computer.
	shutdown(true)
}
