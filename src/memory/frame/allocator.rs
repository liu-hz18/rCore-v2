//! 提供帧（物理页）分配器 [`FRAME_ALLOCATOR`](FrameAllocator)
//!
//! 返回的 [`FrameTracker`] 类型代表一个帧，它在被 drop 时会自动将空间补回分配器中。

// 封装一个物理页分配器
// 这个分配器将不涉及任何的具体算法，具体的算法将用一个名为 Allocator 的 Rust trait 封装起来
// 我们的 FrameAllocator 会依赖于具体的 trait 实现例化

use super::*;
use crate::memory::*;
use algorithm::*;
use lazy_static::*;
use spin::Mutex;

// 我们使用了 lazy_static! 和 Mutex 来包装分配器, 对于 static mut 类型的修改操作是 unsafe 的
// 对于静态全局数据，所有的线程都能访问。当一个线程正在访问这段数据的时候，如果另一个线程也来访问，就可能会产生冲突，并带来难以预测的结果。
// 使用 spin::Mutex<T> 给这段数据加一把锁，一个线程试图通过 lock() 打开锁来获取内部数据的可变引用，如果钥匙被别的线程所占用，那么这个线程就会一直卡在这里
// 
lazy_static! {
    /// 帧分配器，是一种共享*资源*，需要加锁
    /// 默认 FrameAllocator： AllocatorImpl， 即 StackedAllocator
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocatorImpl>> = Mutex::new(FrameAllocator::new(Range::from(
            PhysicalPageNumber::ceil(PhysicalAddress::from(*KERNEL_END_ADDRESS))..PhysicalPageNumber::floor(MEMORY_END_ADDRESS),
        )
    ));
}

/// 基于线段树的帧分配 / 回收
pub struct FrameAllocator<T: Allocator> {
    /// 可用区间的起始
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}

impl<T: Allocator> FrameAllocator<T> {
    /// 创建对象
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy) -> Self {
        FrameAllocator {
            start_ppn: range.into().start,
            allocator: T::new(range.into().len()),
        }
    }

    /// 分配帧，如果没有剩余则返回 `Err`
    pub fn alloc(&mut self) -> MemoryResult<FrameTracker> {
        self.allocator
            .alloc()
            .ok_or("no available frame to allocate")
            .map(|offset| FrameTracker(self.start_ppn + offset))
    }

    /// 将被释放的帧添加到空闲列表的尾部
    ///
    /// 这个函数会在 [`FrameTracker`] 被 drop 时自动调用，不应在其他地方调用
    pub(super) fn dealloc(&mut self, frame: &FrameTracker) {
        self.allocator.dealloc(frame.page_number() - self.start_ppn);
    }
}
