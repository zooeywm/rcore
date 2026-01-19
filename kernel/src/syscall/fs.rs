use config::fd::STDOUT;

use crate::print;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
	match fd {
		STDOUT => {
			let slice = unsafe { core::slice::from_raw_parts(buf, len) };
			let str = core::str::from_utf8(slice).expect("sys_write not utf8 string");
			print!("{}", str);
			len as isize
		}
		_ => {
			panic!("Unsupported fd in sys_write!");
		}
	}
}
