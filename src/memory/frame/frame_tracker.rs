//! 提供物理页的「`Box`」: [`FrameTracker`]

use crate::memory::{address::*, FRAME_ALLOCATOR, PAGE_SIZE};
/// 分配出的物理页
///
/// # `Tracker` 是什么？
/// 太长不看
/// > 可以理解为 [`Box`](alloc::boxed::Box)，而区别在于，其空间不是分配在堆上，
/// > 而是直接在内存中划一片（一个物理页）。
///
/// 在我们实现操作系统的过程中，会经常遇到「指定一块内存区域作为某种用处」的情况。
/// 此时，我们说这块内存可以用，但是因为它不在堆栈上，Rust 编译器并不知道它是什么，所以
/// 我们需要 unsafe 地将其转换为 `&'static mut T` 的形式（`'static` 一般可以省略）。
///
/// 但是，比如我们用一块内存来作为页表，而当这个页表我们不再需要的时候，就应当释放空间。
/// 我们其实更需要一个像「创建一个有生命期的对象」一样的模式来使用这块内存。因此，
/// 我们不妨用 `Tracker` 类型来封装这样一个 `&'static mut` 引用。
///
/// 使用 `Tracker` 其实就很像使用一个 smart pointer。如果需要引用计数，
/// 就在外面再套一层 [`Arc`](alloc::sync::Arc) 就好

// 物理页分配和回收
// 我们需要实现一个分配器可以分配和回收物理页
// 注意到，物理页实际上是一块连续的内存区域，这里我们只是把 内存区域的【起始】物理地址 封装到了一个 FrameTracker 里面。

pub struct FrameTracker(pub(super) PhysicalPageNumber); // 单位是 页号

// 我们设计的初衷是分配器分配给我们 FrameTracker 作为*一个帧*的标识，而随着不再需要这个物理页，我们需要回收，
// 我们利用 Rust 的 drop 机制在析构的时候自动实现回收。
// 每个帧都有一个 FrameTracker !
impl FrameTracker { // 不是内存中4KB的Frame！而只是监督被分配出的一个物理页
    /// 帧的物理地址
    pub fn address(&self) -> PhysicalAddress {
        self.0.into()
    }
    /// 帧的物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0
    }
}

/// `FrameTracker` 可以 deref 得到对应的 `[u8; PAGE_SIZE]`
impl core::ops::Deref for FrameTracker {
    type Target = [u8; PAGE_SIZE];
    fn deref(&self) -> &Self::Target {
        self.page_number().deref_kernel()
    }
}

/// `FrameTracker` 可以 deref 得到对应的 `[u8; PAGE_SIZE]`
impl core::ops::DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.page_number().deref_kernel()
    }
}

/// 帧在释放时会放回 [`static@FRAME_ALLOCATOR`] 的空闲链表中
impl Drop for FrameTracker {
    fn drop(&mut self) {
        //println!("FrameTracker Dropped.");
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
