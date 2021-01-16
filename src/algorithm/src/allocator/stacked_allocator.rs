//! 提供栈结构实现的分配器 [`StackedAllocator`]

use super::Allocator;
use alloc::{vec, vec::Vec};

/// 使用栈结构实现分配器
///
/// 在 `Vec` 末尾进行加入 / 删除。
/// 每个元素 tuple `(start, end)` 表示 [start, end) 区间为可用。
pub struct StackedAllocator {
    list: Vec<(usize, usize)>,
}

// 分配的粒度是 一个页面，每次调用alloc会分配 一个页面，只有*1*个页面!!!
// 分配和回收的 时间复杂度的O(1)
// 空间复杂度O(n). 如果有n个页面，那么最坏情况下，空闲链表中要存储n项
// 最坏情况下，每个物理页需要 16Byte = 2 * 64bit 的存储空间, 也更容易产生内存碎片(最坏时相邻被分配的页面都不是连续的)
// 所以可以采用线段树算法, 优化存储空间(但是会相应提高分配的时间复杂度 -> O(logn))
// 也可以修改 StackedAllocator 使得其使用未被分配的页面空间 (而不是变量list) 来存放页面使用状态。
impl Allocator for StackedAllocator {
    fn new(capacity: usize) -> Self {
        Self {
            list: vec![(0, capacity)], // 初始化该 StackedAllocator 总共可以管理的空间
        }
    }

    fn alloc(&mut self) -> Option<usize> { // O(1)
        if let Some((start, end)) = self.list.pop() { // 栈中弹出一个值
            if end - start > 1 { // 申请粒度是1，如果空间 > 1，我们并不需要这么多，-1后再压回去
                self.list.push((start + 1, end)); // start+1再重新压入栈，可分配空间-1
            }
            Some(start) // 分配成功
        } else {
            None // 分配失败
        }
    }

    fn dealloc(&mut self, index: usize) { // O(1)
        self.list.push((index, index + 1)); // index开始的空间一定是[index, index+1), 所以直接压回即可（因为申请的粒度也是1，所以可以直接放在栈顶，下次也只需要分配栈顶这个）
    }
}
