# rCore 内核源码分析

## 总体架构

这是一个面向 RISC-V 64 位架构的教学型操作系统内核，运行在 QEMU 虚拟机上。内核采用批处理多道程序设计，支持应用预加载、时间片轮转调度和基础系统调用。

**关键特性**：
- `#![no_std]` / `#![no_main]`：裸机环境
- 批处理多任务系统（最多 16 个应用）
- 抢占式调度（100 Hz 定时器中断）
- 两层上下文切换机制（TrapContext + TaskContext）
- 6 个系统调用（5 个已实现）
- 完整的内存管理抽象（但未启用虚拟内存）

---

## 模块结构

```
kernel/src/
├── asm/           # 汇编入口点
│   ├── entry.asm      # _start → rust_main
│   └── link_app.S     # 应用链接脚本
├── boards/        # 板级配置
│   ├── mod.rs
│   └── qemu.rs        # QEMU 平台内存/时钟配置
├── memory/        # 内存管理（已实现但未集成）
│   ├── address.rs      # 虚拟/物理地址类型
│   ├── frame_allocator.rs  # 物理页帧分配器
│   ├── heap_allocator.rs   # 堆分配器（buddy 系统）
│   ├── page_table.rs   # RISC-V SV39 页表
│   └── mod.rs
├── sync/          # 同步原语
│   ├── mod.rs
│   └── up.rs           # UPSafeCell（单核内部可变性）
├── syscall/       # 系统调用
│   ├── mod.rs          # 分发器
│   ├── fs.rs           # 文件系统调用（write）
│   ├── process.rs      # 进程调用（exit, yield）
│   └── time.rs         # 时间调用（nanosleep, gettimeofday）
├── task/          # 任务管理
│   ├── mod.rs          # TaskManager, 调度器
│   ├── context.rs      # TaskContext（内核级上下文）
│   ├── switch.rs       # __switch Rust 封装
│   └── switch.S        # __switch 汇编实现
├── trap/          # 陷阱处理
│   ├── mod.rs          # trap_handler 分发器
│   ├── context.rs      # TrapContext（完整寄存器状态）
│   └── trap.S          # __alltraps/__restore
├── config.rs       # 全局配置常量
├── console.rs      # 控制台输出（基于 SBI）
├── lang_items.rs  # panic 处理器
├── loader.rs      # 应用加载器
├── log.rs         # 日志宏（通过 console 输出）
├── main.rs        # 内核入口
├── sbi.rs         # SBI 封装（定时器、关机）
└── stack_trace.rs # 堆栈跟踪
```

---

## 启动流程

### 1. 硬件入口 (asm/entry.asm)

```assembly
.section .text.entry
.globl _start
_start:
    la sp, boot_stack_top      # 设置启动栈指针
    call rust_main              # 跳转到 Rust 入口
```

### 2. 内核初始化 (main.rs:28-59)

```rust
pub fn rust_main() -> ! {
    // 1. 清零 BSS 段
    (sbss..ebss).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    
    // 2. 输出内存布局
    trace!("rcore started!");
    trace!("text [{:#x}, {:#x})", stext, etext);
    // ...
    
    // 3. 初始化陷阱处理（设置 stvec = __alltraps）
    trap::init();
    
    // 4. 加载应用（从内核数据段复制到固定地址）
    loader::load_apps();
    
    // 5. 启用定时器中断，设置首次触发
    trap::enable_timer_interrupt();
    sbi::set_next_trigger();
    
    // 6. 启动第一个任务（永不返回）
    task::run_first_task();
}
```

### 3. 应用加载 (loader.rs:37-60)

- 从 `_num_app` 符号获取应用数量和地址范围
- 每个应用分配 `APP_SIZE_LIMIT` (128KB) 空间，基地址从 `0x80400000` 开始
- 从内核 `.text` 段复制应用二进制到目标地址
- 调用 `fence.i` 指令刷新指令缓存
- 为每个应用创建 TrapContext 并推入内核栈

---

## 核心子系统

### 1. 任务管理与调度 (task/)

#### 数据结构

**TaskControlBlock** (mod.rs:7-10)
```rust
pub struct TaskControlBlock {
    pub task_status: TaskStatus,  // UnInit | Ready | Running | Exited
    pub task_cx: TaskContext,      // 内核级上下文（ra, sp, s0-s11）
}
```

**TaskContext** (context.rs:4-11) - 最小上下文，用于内核切换
```rust
pub struct TaskContext {
    ra: usize,          // 返回地址（指向 __restore）
    sp: usize,          // 内核栈指针
    s: [usize; 12],     // 被调用者保存寄存器
}
```

**TrapContext** (trap/context.rs:5-12) - 完整上下文，用于用户态切换
```rust
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],  // x0-x31 通用寄存器
    pub sstatus: Sstatus, // 状态寄存器
    pub sepc: usize,     // 异常返回地址
}
```

#### 调度机制

**轮转调度** (mod.rs:166-172)
- 从 `current_task + 1` 开始查找第一个 Ready 任务
- 循环遍历，保证公平性
- 不考虑优先级

**调度触发时机**
- **定时器中断** (`trap/mod.rs:52-55`)：`SupervisorTimer` → `suspend_current_and_run_next()`
- **系统调用 sys_yield** (`syscall/process.rs:13-16`)：主动让出 CPU
- **系统调用 sys_exit** (`syscall/process.rs:6-10`)：任务退出
- **异常** (`trap/mod.rs:37-51`)：内存故障/非法指令终止任务

#### 上下文切换流程

**首次启动** (mod.rs:109-120)
1. 初始化 Task0 为 Running 状态
2. 调用 `__switch` 从 dummy context 切换到 task0
3. Task0 的 `ra` 指向 `__restore`，`sp` 指向包含 TrapContext 的内核栈
4. `__restore` 恢复 TrapContext，`sret` 返回用户态

**任务切换** (mod.rs:140-161)
1. 查找下一个 Ready 任务
2. `__switch` 保存当前任务上下文到 `task_cx`
3. 恢复下一任务的 `task_cx`（`ra` 指向 `__restore`）
4. `__restore` 从内核栈恢复 TrapContext，`sret` 返回用户态

---

### 2. 陷阱处理 (trap/)

#### 汇编入口 (trap.S)

**__alltraps** - 陷阱入口
```assembly
csrrw sp, sscratch, sp     # 交换 sp 和 sscratch（保存用户栈）
addi sp, sp, -34*8         # 在内核栈分配 TrapContext
SAVE_GP 1-31               # 保存通用寄存器
csrr t0, sstatus           # 保存 sstatus
csrr t1, sepc              # 保存 sepc
csrr t2, sscratch          # 保存用户栈指针
call trap_handler          # 调用 Rust 分发器
```

**__restore** - 陷阱返回
```assembly
ld t0, 32*8(sp)            # 恢复 sstatus
ld t1, 33*8(sp)            # 恢复 sepc
ld t2, 2*8(sp)             # 恢复用户栈指针
csrw sstatus, t0           # 写入 CSR
csrw sepc, t1
csrw sscratch, t2
LOAD_GP 1-31               # 恢复通用寄存器
csrrw sp, sscratch, sp     # 交换栈回用户栈
sret                        # 返回用户态
```

#### 分发器 (trap/mod.rs)

```rust
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;  // 跳过 ecall 指令
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            // a7=x17=syscall_id, a0=x10=返回值, a0-a2=参数
        }
        Trap::Exception(Exception::StoreFault | StorePageFault) => {
            error!("PageFault at {:#x}", stval);
            exit_current_and_run_next();  // 终止应用
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("IllegalInstruction");
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();              // 设置下一次定时器
            suspend_current_and_run_next();  // 时间片切换
        }
        _ => panic!("Unsupported trap: {:?}", scause.cause())
    }
    cx
}
```

---

### 3. 系统调用 (syscall/)

#### 已实现的系统调用

| ID | 名称 | 模块 | 功能 |
|----|------|------|------|
| 64 | WRITE | fs.rs | 写入文件描述符（仅支持 STDOUT） |
| 93 | EXIT | process.rs | 退出当前进程 |
| 101 | NANOSLEEP | time.rs | 高精度休眠（纳秒级） |
| 124 | YIELD | process.rs | 让出 CPU |
| 169 | GETTIMEOFDAY | time.rs | 获取当前时间（微秒级） |
| 140 | SETPRIORITY | - | 仅定义，未实现 |

#### 分发机制 (syscall/mod.rs)

```rust
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        EXIT => sys_exit(args[0] as i32),
        NANOSLEEP => sys_nanosleep(args[0] as *const KernelTimespec, args[1] as *mut KernelTimespec),
        YIELD => sys_yield(),
        GETTIMEOFDAY => sys_gettimeofday(args[0] as *mut TimeVal, args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
```

#### 调用约定

使用 RISC-V ABI：
- `x[10]` (a0)：参数 0 / 返回值
- `x[11]` (a1)：参数 1
- `x[12]` (a2)：参数 2
- `x[17]` (a7)：系统调用 ID

---

### 4. 内存管理 (memory/)

该模块已完整实现但**未集成到主内核**，当前内核使用直接内存操作。

#### 地址抽象 (address.rs)

**虚拟地址类型**
- `VirtAddr(usize)` - 39 位 SV39 虚拟地址
- `VirtPageNum(usize)` - 27 位虚拟页号，支持三级页表索引提取

**物理地址类型**
- `PhysAddr(usize)` - 56 位物理地址
- `PhysPageNum(usize)` - 物理页号，支持转换为 satp 值

#### 物理页帧分配器 (frame_allocator.rs)

采用**栈式分配策略**：
```rust
pub struct StackFrameAllocator {
    current: usize,               // 栈顶指针，[current, end) 未分配
    end: usize,                   // 物理内存上界 (0x88000000)
    recycled: VecDeque<usize>,    // 回收的物理页号栈
}
```

- 优先从 `recycled` 栈顶分配（复用已释放页面）
- 若栈空，则从 `current` 分配并递增
- `Frame` 结构体通过 RAII 自动回收

#### 堆分配器 (heap_allocator.rs)

基于 `buddy_system_allocator`：
```rust
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();
static HEAP_SPACE: [u8; KERNEL_HEAP_SIZE];  // 3 MiB
```
标记为 `#[global_allocator]`，为内核全局分配器。

#### 页表管理 (page_table.rs)

实现 RISC-V SV39 三级页表：
```rust
pub struct PageTable {
    root: PhysPageNum,    // 根页表物理页号
    frames: Vec<Frame>,   // 持有所有页表帧（自动 RAII）
}
```

- `map(vpn, ppn, flags)`：建立映射（懒分配页表）
- `unmap(vpn)`：清除映射
- `translate_virt_addr(va)`：地址翻译
- `token()`：生成 satp 寄存器值
- `read_ref<T>()` / `read_str()`：用户空间访问

---

### 5. 同步原语 (sync/)

#### UPSafeCell (sync/up.rs)

```rust
pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
```

- 单核环境的内部可变性封装
- 绕过编译期借用检查，依赖运行时 Panic
- 使用 `UPSafeCell<T>` 包裹静态 `TaskManager`，实现共享可变状态

---

### 6. 硬件抽象

#### SBI 封装 (sbi.rs)

```rust
pub fn set_next_trigger() {
    set_timer(time::read() as u64 + MTIME_FREQUENCY_HZ / TICKS_PER_SEC);
}

pub fn get_time_us() -> u64 {
    time::read() as u64 / (MTIME_FREQUENCY_HZ / MICRO_PER_SEC);
}

pub fn shutdown(failure: bool) -> ! {
    if failure {
        system_reset(Shutdown, SystemFailure);
    } else {
        system_reset(Shutdown, NoReason);
    }
}
```

- 定时器管理（100 Hz 抢占）
- 时间获取（微秒精度）
- 系统关机/重启

#### 控制台输出 (console.rs)

通过 `sbi_rt::console_write_byte` 实现 `print!` / `println!` 宏。

---

## 配置参数 (config.rs)

| 参数 | 值 | 说明 |
|------|-----|------|
| `MAX_APP_NUM` | 16 | 最大应用数量 |
| `USER_STACK_SIZE` | 8192 (8 KiB) | 用户栈大小 |
| `KERNEL_STACK_SIZE` | 8192 (8 KiB) | 内核栈大小 |
| `APP_BASE_ADDRESS` | 0x80400000 | 应用加载基地址 |
| `APP_SIZE_LIMIT` | 0x20000 (128 KiB) | 单个应用空间限制 |
| `TICKS_PER_SEC` | 100 | 定时器频率（100 Hz） |
| `PAGE_SIZE` | 0x1000 (4 KiB) | 物理页大小 |
| `KERNEL_HEAP_SIZE` | 0x300000 (3 MiB) | 堆大小 |
| `MEMORY_END` | 0x88000000 | 物理内存上界（128 MiB） |
| `MTIME_FREQUENCY_HZ` | 10000000 | mtime 寄存器频率（10 MHz） |

---

## 架构设计特点

### 1. 两层上下文系统

| 上下文类型 | 位置 | 用途 | 保存内容 |
|-----------|------|------|---------|
| **TrapContext** | 内核栈 | 用户态↔内核态切换 | x0-x31, sstatus, sepc（完整状态） |
| **TaskContext** | TCB | 内核级任务切换 | ra, sp, s0-s11（最小集合） |

优势：
- 内核切换只保存 callee-saved 寄存器，减少开销
- 用户态切换保存完整状态，保证执行环境恢复

### 2. 批处理多道程序设计

- 应用在编译时静态链接到内核镜像
- 启动时一次性加载所有应用到固定内存地址
- 时间片轮转调度，无优先级
- 应用崩溃不影响内核（直接终止任务）

### 3. 抢占式调度

- 基于 RISC-V `mtime` 寄存器和 SBI 定时器接口
- 100 Hz 频率（每 10ms 触发一次）
- 定时器中断触发 `suspend_current_and_run_next()`

### 4. 类型安全设计

- 使用 newtype 模式（`VirtAddr`, `PhysAddr`）避免地址混淆
- RAII 自动资源管理（`Frame`, `PageTable` 持有 `Vec<Frame>`）
- 零成本抽象（地址类型仅包装 `usize`）

### 5. 单核假设

- 使用 `UPSafeCell<T>` 绕过 `Sync` 约束
- 无锁设计，依赖单核原子性
- 若需支持多核，需替换为锁/原子操作

---

## 当前状态与扩展方向

### 已实现
- ✅ 批处理多任务管理
- ✅ 抢占式轮转调度
- ✅ 陷阱处理（系统调用、异常、定时器）
- ✅ 5 个基础系统调用（write, exit, yield, nanosleep, gettimeofday）
- ✅ 控制台输出
- ✅ 时间管理
- ✅ 堆栈跟踪

### 部分实现
- ⚠️ 内存管理模块（完整但未集成）
- ⚠️ 仅支持 STDOUT 写入（无文件系统）

### 未实现
- ❌ 虚拟内存（当前使用直接物理地址）
- ❌ 进程管理（fork, exec, wait）
- ❌ 文件系统（open, close, read）
- ❌ 信号机制
- ❌ 多核支持

### 扩展建议
1. **集成虚拟内存**：在任务切换时加载 `PageTable::token()` 到 satp 寄存器
2. **实现文件系统**：添加 open/close/read 系统调用，支持 stdin/stderr
3. **增强进程管理**：实现 fork/exec/wait，支持多进程
4. **添加信号机制**：实现信号发送/处理
5. **多核支持**：将 `UPSafeCell` 替换为锁/原子操作

---

