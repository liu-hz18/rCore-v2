// 我们可以直接开一个静态的 8M 数组作为堆的空间，然后调用 @jiege 开发的 Buddy System Allocator。

/// 操作系统动态分配内存所用的堆大小（8M）
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

