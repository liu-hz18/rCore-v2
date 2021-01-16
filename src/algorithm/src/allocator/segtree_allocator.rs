//! 使用 线段树 进行页式内存管理，每次分配一个页面

use super::Allocator;
use alloc::{vec, vec::Vec};

// 空间复杂度有所下降，每个物理页面占用 固定 2bit 空间, 大大减小
// 时间复杂度上升，从O(1) -> O(logn)

// 对于每个块，都有左右指针指向其管理的内存地址 [left, right)
// 内存分配实际上就是区间查询出满足大小要求的内存块，并进行区间更新，将内存块更新为已占用
// 内存释放实际上就是进行区间更新操作，将内存块更新为未使用

// 内存释放: 更新父区间的字段状态 => 更新左子区间的字段状态 + 更新右子区间的字段状态
// 内存分配: 需要从一个连续区间中查询满足大小要求的内存块。

// 但本例中只需要支持单个页面分配，也就是支持 单点查询 和 单点修改 就可以了

pub struct SegmentTreeAllocator {
    tree: Vec<bool>, // 最坏 4n bit. 但是考虑到capacity一般是2^n, 实际上只需要2n bit
    non_leaf_size: usize,
}

impl SegmentTreeAllocator {
    // 向上更新祖先节点，直到根
    fn update_node(&mut self, mut index: usize, value: bool) {
        self.tree[index] = value;
        while index > 1 {
            index /= 2;
            let v = self.tree[index * 2] && self.tree[index * 2 + 1];
            self.tree[index] = v;
        }
    }
}

impl Allocator for SegmentTreeAllocator {
    fn new(capacity: usize) -> Self {
        let leaf_count = capacity.next_power_of_two(); // 扩展到2^n, 返回 min 2^k s.t. 2^k>v-1
        let mut tree = vec![false; 2 * leaf_count]; // 2叉树开2倍内存
        for i in capacity..leaf_count {
            tree[leaf_count + i] = false; // 所有叶子节点置0, 表示空闲
        }
        // 根节点是 tree[1]. 左孩子 2*i, 右孩子2*i+1
        for i in (1..leaf_count).rev() {
            let v = tree[i * 2] && tree[i * 2 + 1]; // 更新父节点, 两个子节点中有1个是0，则父节点为0
            tree[i] = v;
        }
        Self { tree: tree, non_leaf_size: leaf_count }
    }

    fn alloc(&mut self) -> Option<usize> {
        if self.tree[1] == true {
            None // tree is full
        } else {
            let mut node_index = 1;
            // search for a false leaf
            while node_index < self.non_leaf_size { // 在所有非叶子节点中遍历，直到一个叶子节点
                if self.tree[node_index*2] == false { // 左节点空闲，以左节点为根，再次迭代
                    node_index *= 2;
                } else if self.tree[node_index*2+1] == false { // 同理
                    node_index = node_index*2 + 1;
                } else { // 左右子节点都不空闲
                    return None;
                }
            }
            assert!(!self.tree[node_index]);
            self.update_node(node_index, true);
            Some(node_index - self.non_leaf_size)
        }
    }

    fn dealloc(&mut self, index: usize) {
        let node_index = index + self.non_leaf_size;
        assert!(self.tree[node_index]);
        self.update_node(node_index, false);
    }
}
