//! 提供向量分配器的简单实现 [`BitmapVectorAllocator`]

use super::VectorAllocator;
use bit_field::BitArray;
use core::cmp::min;

/// Bitmap 中的位数（4K bit）
const BITMAP_SIZE: usize = 4096;

/// 向量分配器的简单实现，每字节用一位表示
pub struct BitmapVectorAllocator {
    /// 容量，单位为 bitmap 中可以使用的位数，即待分配空间的 字节数
    capacity: usize,
    /// 每一位 0 表示空闲
    bitmap: [u8; BITMAP_SIZE / 8], // 最多支持512Byte的分配管理
}

impl VectorAllocator for BitmapVectorAllocator {
    fn new(capacity: usize) -> Self { // 单位是 字节
        Self {
            capacity: min(BITMAP_SIZE, capacity),
            bitmap: [0u8; BITMAP_SIZE / 8],
        }
    }
    // O(capacity / align) ~ O(capacity)
    fn alloc(&mut self, size: usize, align: usize) -> Option<usize> { // size, align 单位都是字节
        for start in (0..self.capacity - size).step_by(align) {
            if (start..start + size).all(|i| !self.bitmap.get_bit(i)) { // [start, start+size)中的所有字节都没有被占用，则该内存块可分配
                (start..start + size).for_each(|i| self.bitmap.set_bit(i, true)); // 标记置为1
                return Some(start); // 分配成功
            }
        }
        None // 分配失败
    }
    fn dealloc(&mut self, start: usize, size: usize, _align: usize) { // O(size)
        assert!(self.bitmap.get_bit(start)); // 首先是一个已经被分配的页面
        (start..start + size).for_each(|i| self.bitmap.set_bit(i, false)); // [start, start+size)的字节标记清零
    }
}
