//! 实现操作系统动态内存分配所用的堆
//!
//! 基于 `buddy_system_allocator` crate，致敬杰哥。
use super::config::KERNEL_HEAP_SIZE; // 0x80_0000
use buddy_system_allocator::LockedHeap;

/// 进行动态内存分配所用的堆空间
/// 
/// 大小为 [`KERNEL_HEAP_SIZE`] , 类型为[u8; KERNEL_HEAP_SIZE] 
/// 这段空间编译后会被放在操作系统执行程序的 bss 段
/// 我们具有随意使用内存空间的权力，因此我们可以在内存中随意划一段空间，然后用相应的算法来实现一个堆。
/// 但是，在代码中用全局变量来表示堆并将其放在 .bss 字段，是一个很简单的实现：
///   这样堆空间就包含在内核的二进制数据之中了，而自 KERNEL_END_ADDRESS 以后的空间就都可以给进程使用。
/// 注意堆空间只能用基本数据类型, [u8], 不能用Vec
/// 因为 Vec 的内存分配依赖于 操作系统，而操作系统又会使用 Vec 分配堆内存，这样程序就会陷入一个循环。(依然可以编译)
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// 堆，动态内存分配器
/// 不造轮子！
/// ### `#[global_allocator]`
/// [`LockedHeap`] 实现了 [`alloc::alloc::GlobalAlloc`] trait，
/// 可以为全局需要用到堆的地方分配空间。例如 `Box` `Arc` 等
#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

/// 初始化操作系统运行时堆空间
pub fn init() {
    // 告诉分配器使用这一段预留的空间作为堆
    unsafe {
        HEAP.lock().init(
            HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE
        )
    }
}

/// 空间分配错误的回调，直接 panic 退出
#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
