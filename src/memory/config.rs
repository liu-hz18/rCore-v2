//! 定义一些内存相关的常量
use super::address::*;
use lazy_static::*;

/// 页 / 帧大小，必须是 2^n
pub const PAGE_SIZE: usize = 4096;

/// 可以访问的内存区域起始地址
pub const MEMORY_START_ADDRESS: PhysicalAddress = PhysicalAddress(0x8000_0000);
/// 可以访问的内存区域结束地址
pub const MEMORY_END_ADDRESS: PhysicalAddress = PhysicalAddress(0x8800_0000);


// 我们直接将 DRAM 物理内存结束地址硬编码到内核中，
// 同时因为我们操作系统本身也用了一部分空间，我们也记录下操作系统用到的地址结尾（即 linker script 中的 kernel_end）。
lazy_static! { // lazy_static! 宏帮助我们在第一次使用 lazy_static! 宏包裹的变量时自动完成这些求值工作。
    /// 内核代码结束的地址，即可以用来分配的内存起始地址
    ///
    /// 因为 Rust 语言限制，我们只能将其作为一个运行时求值的 static 变量，而不能作为 const
    pub static ref KERNEL_END_ADDRESS: PhysicalAddress = PhysicalAddress(kernel_end as usize);
}

// 我们通过划分出一段静态内存为操作系统实现了动态内存的分配
// 我们可以直接开一个静态的 8M 数组作为堆的空间，然后调用 @jiege 开发的 Buddy System Allocator。
/// 操作系统动态分配内存所用的堆大小（8M）
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

extern "C" {
    /// 由 `linker.ld` 指定的内核代码结束位置
    ///
    /// 作为变量存在 [`static@KERNEL_END_ADDRESS`]
    fn kernel_end();
}
