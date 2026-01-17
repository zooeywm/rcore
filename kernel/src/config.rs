/// Mtime register frequency in us
pub const MTIME_FREQUENCY_HZ: u64 = 10_000_000;

// Batch
pub const MAX_APP_NUM: usize = 16;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
