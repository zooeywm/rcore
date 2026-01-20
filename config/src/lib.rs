#![no_std]

/// Syscall
pub mod syscall {
	pub const WRITE: usize = 64;
	pub const EXIT: usize = 93;
	pub const NANOSLEEP: usize = 101;
	pub const YIELD: usize = 124;
	pub const GETTIMEOFDAY: usize = 169;
	pub const SETPRIORITY: usize = 140;

	#[repr(C)]
	pub struct KernelTimespec {
		pub tv_sec:  i64,
		pub tv_nsec: i64,
	}

	impl KernelTimespec {
		pub fn new(tv_sec: i64, tv_nsec: i64) -> Self { Self { tv_sec, tv_nsec } }

		pub fn sec(tv_sec: i64) -> Self { Self { tv_sec, tv_nsec: 0 } }

		pub fn nsec(tv_nsec: i64) -> Self { Self { tv_sec: 0, tv_nsec } }
	}

	#[repr(C)]
	#[derive(Debug, Default)]
	pub struct TimeVal {
		pub sec:  u64,
		pub usec: u64,
	}
	impl TimeVal {
		pub fn new() -> Self { Self::default() }
	}
}

/// Fd
pub mod fd {
	pub const STDOUT: usize = 1;
}

pub mod errno {
	pub const EINVAL: isize = 22;
}
