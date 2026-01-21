//! Constants used in rCore for qemu

/// Mtime 寄存器频率（微秒）
pub const MTIME_FREQUENCY_HZ: u64 = 10_000_000;
/// 物理地址起始于`0x8000_0000`，我们现在有100M内存
pub const MEMORY_END: usize = 0x8800_0000;
