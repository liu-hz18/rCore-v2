//! 一个线程中关于内存空间的所有信息 [`MemorySet`]
//!

// 我们需要把内核的每个段根据不同的属性写入上面的封装的 Mapping 中，并把它作为一个新的结构 MemorySet 给后面的线程的概念使用
// 每个线程（到目前为止你可以大致理解为自己电脑中的同时工作的应用程序们）将会拥有一个 MemorySet
// 其中存的将会是「它看到的虚拟内存空间分成的内存段」和「这些段中包含的虚拟页到物理页的映射」

use crate::memory::{
    address::*,
    config::*,
    mapping::{Flags, MapType, Mapping, Segment},
    range::Range,
    MemoryResult,
};
use alloc::{vec, vec::Vec};

/// 一个进程所有关于内存空间管理的信息
pub struct MemorySet {
    /// 维护页表和映射关系
    pub mapping: Mapping,
    /// 每个字段
    pub segments: Vec<Segment>,
}

impl MemorySet {
    /// 创建内核重映射
    pub fn new_kernel() -> MemoryResult<MemorySet> {
        // 在 linker.ld 里面标记的各个字段的起始点，均为 4K 对齐
        extern "C" {
            fn text_start();
            fn rodata_start();
            fn data_start();
            fn bss_start();
        }

        // 建立字段
        let segments = vec![
            // .text 段，r-x
            Segment {
                map_type: MapType::Linear,
                range: Range::from((text_start as usize)..(rodata_start as usize)),
                flags: Flags::READABLE | Flags::EXECUTABLE,
            },
            // .rodata 段，r--
            Segment {
                map_type: MapType::Linear,
                range: Range::from((rodata_start as usize)..(data_start as usize)),
                flags: Flags::READABLE,
            },
            // .data 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from((data_start as usize)..(bss_start as usize)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // .bss 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(VirtualAddress::from(bss_start as usize)..*KERNEL_END_ADDRESS),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // 剩余内存空间，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(*KERNEL_END_ADDRESS..VirtualAddress::from(MEMORY_END_ADDRESS)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
        ];
        let mut mapping = Mapping::new()?;

        // 每个字段在页表中进行映射
        for segment in segments.iter() {
            mapping.map(segment, None)?;
        }
        Ok(MemorySet {
            mapping,
            segments,
        })
    }

    /// 替换 `satp` 以激活页表
    ///
    /// 如果当前页表就是自身，则不会替换，但仍然会刷新 TLB。
    pub fn activate(&self) {
        self.mapping.activate();
    }

    /// 添加一个 [`Segment`] 的内存映射
    pub fn add_segment(&mut self, segment: Segment, init_data: Option<&[u8]>) -> MemoryResult<()> {
        // 检测 segment 没有重合
        assert!(!self.overlap_with(segment.page_range()));
        // 映射
        self.mapping.map(&segment, init_data)?;
        self.segments.push(segment);
        Ok(())
    }

    /// 移除一个 [`Segment`] 的内存映射
    ///
    /// `segment` 必须已经映射
    pub fn remove_segment(&mut self, segment: &Segment) -> MemoryResult<()> {
        // 找到对应的 segment
        let segment_index = self
            .segments
            .iter()
            .position(|s| s == segment)
            .expect("segment to remove cannot be found");
        self.segments.remove(segment_index);
        // 移除映射
        self.mapping.unmap(segment);
        Ok(())
    }

    /// 检测一段内存区域和已有的是否存在重叠区域
    pub fn overlap_with(&self, range: Range<VirtualPageNumber>) -> bool {
        for seg in self.segments.iter() {
            if range.overlap_with(&seg.page_range()) {
                return true;
            }
        }
        false
    }
}
