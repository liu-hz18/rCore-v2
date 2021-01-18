//! virtio MMIO 总线协议驱动
//!
//! 目前仅仅实现了 virtio Block Device 协议，另外还有类似 virtio Network 等协议

// virtio 起源于 virtio: Towards a De-Facto Standard For Virtual I/O Devices 这篇论文，主要针对于半虚拟化技术中对通用设备的抽象。
// 以 virtio 为中心的总线下又挂载了 virtio-blk（块设备）总线、virtio-net（网络设备）总线、virtio-pci（PCI 设备）总线等，本身就构成一个设备树。

use super::super::block::virtio_blk;
use crate::memory::{
    frame::{FrameTracker, FRAME_ALLOCATOR},
    mapping::Mapping,
    PhysicalAddress, VirtualAddress, PAGE_SIZE,
};
use alloc::collections::btree_map::BTreeMap;
use device_tree::{util::SliceRead, Node};
use lazy_static::lazy_static;
use spin::RwLock;
use virtio_drivers::{DeviceType, VirtIOHeader}; // 我们使用 rCore 中的 virtio_drivers 库，这个库会帮我们通过 MMIO 的方式对设备进行交互，同时我们也需要给这个库提供一些诸如申请物理内存、物理地址虚拟转换等接口。

/// 从设备树的某个节点探测 virtio 协议具体类型
/// 从设备树节点的 reg 信息中可以读出设备更详细信息的放置位置（如：在 0x10000000 - 0x10010000 ），
/// 这段区间虽然算是内存区间, 但是我们的物理内存只分布在 0x80000000 到 0x88000000 的空间中
/// 这就是MMIO，总线把对设备操作信息传递也映射成了内存的一部分，CPU 操作设备和访问内存的形式没有任何的区别，但读写效果是不同的
/// 为了访问这段地址，我们也需要把它加到页表里面
pub fn virtio_probe(node: &Node) {
    // reg 属性中包含了描述设备的 Header 的位置
    let reg = match node.prop_raw("reg") {
        Some(reg) => reg,
        _ => return,
    };
    let pa = PhysicalAddress(reg.as_slice().read_be_u64(0).unwrap() as usize);
    let va = VirtualAddress::from(pa);
    let header = unsafe { &mut *(va.0 as *mut VirtIOHeader) };
    // 目前只支持某个特定版本的 virtio 协议
    if !header.verify() {
        return;
    }
    // 判断设备类型
    match header.device_type() {
        DeviceType::Block => virtio_blk::add_driver(header),
        device => println!("unrecognized virtio device: {:?}", device),
    }
}

lazy_static! {
    /// 用于放置给设备 DMA 所用的物理页（[`FrameTracker`]）
    pub static ref TRACKERS: RwLock<BTreeMap<PhysicalAddress, FrameTracker>> =
        RwLock::new(BTreeMap::new());
}

// 需要申请物理内存、物理地址虚拟转换等接口的原因:
// 本身设备是通过直接内存访问DMA（Direct Memory Access）技术来实现数据传输的
// CPU 只需要给出要传输哪些内容，放在哪段物理内存上面，把请求告诉设备，设备后面的操作就会利用 DMA 而不经过 CPU 直接传输
// 在传输结束之后，会通过中断请求 IRQ（Interrupt ReQuest）技术沿着设备树把"我做完了"这个信息告诉 CPU
// CPU 会作为一个中断进一步处理
// 为了实现 DMA，我们需要一些请求和内存空间，比如让磁盘把数据传到某个内存段，我们需要告诉设备内存的物理地址（之所以不是虚拟地址是因为 DMA 不会经过 CPU 的 MMU 技术）
// 而且这个物理地址最好是连续的
// 同时，我们在栈上申请一个请求的结构，这个结构的物理地址也要告诉设备，所以也需要一些虚实地址转换的接口。

/// 为 DMA 操作申请连续 pages 个物理页（为 [`virtio_drivers`] 库提供）
///
/// 为什么要求连续的物理内存？设备的 DMA 操作只涉及到内存和对应设备
/// 这个过程不会涉及到 CPU 的 MMU 机制，我们只能给设备传递物理地址
/// 而陷于我们之前每次只能分配一个物理页的设计，这里我们假设我们连续分配的地址是连续的
/// 我们的 FRAME_ALLOCATOR 还只能分配一个帧出来，我们连续调用，暂时先假设他是连续的。
#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysicalAddress {
    let mut pa: PhysicalAddress = Default::default();
    let mut last: PhysicalAddress = Default::default();
    for i in 0..pages {
        let tracker: FrameTracker = FRAME_ALLOCATOR.lock().alloc().unwrap();
        if i == 0 {
            pa = tracker.address();
        } else {
            assert_eq!(last + PAGE_SIZE, tracker.address());
        }
        last = tracker.address();
        TRACKERS.write().insert(last, tracker);
    }
    pa
}

/// 为 DMA 操作释放对应的之前申请的连续的物理页（为 [`virtio_drivers`] 库提供）
#[no_mangle]
extern "C" fn virtio_dma_dealloc(pa: PhysicalAddress, pages: usize) -> i32 {
    for i in 0..pages {
        TRACKERS.write().remove(&(pa + i * PAGE_SIZE));
    }
    0
}

/// 将物理地址转为虚拟地址（为 [`virtio_drivers`] 库提供）
///
/// 需要注意，我们在 0xffffffff80200000 到 0xffffffff88000000 是都有对应的物理地址映射的
/// 因为在内核重映射的时候，我们已经把全部的段放进去了
/// 所以物理地址直接加上 Offset 得到的虚拟地址是可以通过任何内核进程的页表来访问的
#[no_mangle]
extern "C" fn virtio_phys_to_virt(pa: PhysicalAddress) -> VirtualAddress {
    VirtualAddress::from(pa)
}

/// 将虚拟地址转为物理地址（为 [`virtio_drivers`] 库提供）
///
/// 需要注意，实现这个函数的目的是告诉 DMA 具体的请求，请求在实现中会放在栈上面
/// 而在我们的实现中，栈是以 Framed 的形式分配的，并不是高地址的线性映射 Linear
/// 为了得到正确的物理地址并告诉 DMA 设备，我们只能查页表
#[no_mangle]
extern "C" fn virtio_virt_to_phys(va: VirtualAddress) -> PhysicalAddress {
    Mapping::lookup(va).unwrap()
}
