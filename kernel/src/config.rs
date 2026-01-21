// Batch
pub const MAX_APP_NUM: usize = 16;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;

// Preemptive
pub const TICKS_PER_SEC: u64 = 100;
pub const MICRO_PER_SEC: u64 = 1_000_000;

/// 物理页大小，十六进制表示方便地址转页号的计算(2^12=4096=0x1000)
pub const PAGE_SIZE: usize = 0x1000;
/// 物理页内寻址的位数
pub const PAGE_SIZE_BITS: usize = 12;

/// 内核堆大小
pub const KERNEL_HEAP_SIZE: usize = 0x300000;

pub use crate::boards::qemu::{MEMORY_END, MTIME_FREQUENCY_HZ};
