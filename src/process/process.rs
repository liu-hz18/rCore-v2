//! 进程 [`Process`]
// 在我们实现的简单操作系统中，进程只需要维护页面映射，并且存储一点额外信息
// 
use super::*;
use crate::fs::*;
use xmas_elf::ElfFile;
use alloc::{vec, vec::Vec};

/// 进程的信息
pub struct Process {
    /// 是否属于用户态
    pub is_user: bool, // 用户态标识：我们会在后面进行区分内核态线程和用户态线程。
    /// 用 `Mutex` 包装一些可变的变量
    pub inner: Mutex<ProcessInner>, // 进程也需要一部分是可变的。
}

pub struct ProcessInner {
    /// 进程中的线程公用页表 / 内存映射
    pub memory_set: MemorySet, // 访存空间. ：进程中的线程会共享同一个页表，即可以访问的虚拟内存空间
    // 打开的文件描述符
    pub descriptors: Vec<Arc<dyn INode>>,
}

#[allow(unused)]
impl Process {
    /// 创建一个内核进程, 只能创建一个内核进程！！！
    pub fn new_kernel() -> MemoryResult<Arc<Self>> {
        Ok(Arc::new(Self {
            is_user: false,
            inner: Mutex::new(ProcessInner {
                memory_set: MemorySet::new_kernel()?,
                descriptors: vec![STDIN.clone(), STDOUT.clone()], // 目前只支持打开STDIN和STDOUT
            }),
        }))
    }

    /// 创建进程，从文件中读取代码, 用户进程根据文件创建
    pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<Arc<Self>> {
        Ok(Arc::new(Self {
            is_user,
            inner: Mutex::new(ProcessInner {
                memory_set: MemorySet::from_elf(file, is_user)?,
                descriptors: vec![STDIN.clone(), STDOUT.clone()],
            }),
        }))
    }

    /// 上锁并获得可变部分的引用
    pub fn inner(&self) -> spin::MutexGuard<ProcessInner> {
        self.inner.lock()
    }

    /// 分配一定数量的连续虚拟空间
    ///
    /// 从 `memory_set` 中找到一段给定长度的未占用虚拟地址空间，分配物理页面并建立映射。返回对应的页面区间。
    /// 返回的是地址区间
    /// `flags` 只需包括 rwx 权限，user 位会根据进程而定。
    pub fn alloc_page_range(
        &self,
        size: usize,
        flags: Flags,
    ) -> MemoryResult<Range<VirtualAddress>> {
        let memory_set = &mut self.inner().memory_set;

        // memory_set 只能按页分配，所以让 size 向上取整页
        let alloc_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        // 从 memory_set 中找一段不会发生重叠的空间
        let mut range = Range::<VirtualAddress>::from(0x1000000..0x1000000 + alloc_size);
        while memory_set.overlap_with(range.into()) {
            range.start += alloc_size;
            range.end += alloc_size;
        }
        // 分配物理页面，建立映射
        memory_set.add_segment(
            Segment {
                map_type: MapType::Framed,
                range,
                flags: flags | Flags::user(self.is_user),
            },
            None,
        )?;
        // 返回地址区间（使用参数 size，而非向上取整的 alloc_size）
        Ok(Range::from(range.start..(range.start + size)))
    }
}
