//! 设备树读取
//!
//! 递归遍历设备树并初始化

use super::bus::virtio_mmio::virtio_probe;
use crate::memory::VirtualAddress;
use core::slice;
use device_tree::{DeviceTree, Node};

/// 验证某内存段为设备树格式的 Magic Number（固定）
const DEVICE_TREE_MAGIC: u32 = 0xd00d_feed;

/// 递归遍历设备树
/// 遍历过程中，一旦发现了一个支持 "virtio,mmio" 的设备（其实就是 QEMU 模拟的存储设备），就进入下一步加载驱动的逻辑。
fn walk(node: &Node) {
    // 检查设备的协议支持并初始化
    if let Ok(compatible) = node.prop_str("compatible") {
        if compatible == "virtio,mmio" {
            virtio_probe(node);
        }
    }
    // 遍历子树
    for child in node.children.iter() {
        walk(child);
    }
}

/// 整个设备树的 Headers（用于验证和读取）
struct DtbHeader {
    magic: u32,
    size: u32,
}

/// 遍历设备树并初始化设备
pub fn init(dtb_va: VirtualAddress) {
    let header = unsafe { &*(dtb_va.0 as *const DtbHeader) };
    // from_be 是大小端序的转换（from big endian）
    // 有一步来验证 Magic Number，这一步是一个保证系统可靠性的要求，是为了验证这段内存到底是不是设备树。
    let magic = u32::from_be(header.magic);
    if magic == DEVICE_TREE_MAGIC {
        let size = u32::from_be(header.size);
        // 拷贝数据，加载并遍历
        let data = unsafe { slice::from_raw_parts(dtb_va.0 as *const u8, size as usize) };
        if let Ok(dt) = DeviceTree::load(data) {
            walk(&dt.root);
        }
    }
}
