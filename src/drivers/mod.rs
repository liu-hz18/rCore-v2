//! 驱动模块
//!
//! 负责驱动管理

// 设计模式（从上往下）
// trait BlockDevice
// lockDevice(pub Arc Driver)
// trait Driver // Driver 作为一个核心 trait 为上提供实现，上层也就是 Driver 的使用侧（设备的抽象），而下层则是 Driver 的实现侧（设备的实现）
// struct VirtioBlkDriver

use crate::memory::{PhysicalAddress, VirtualAddress};

pub mod block;
pub mod bus;
pub mod device_tree;
pub mod driver;

/// 从设备树的物理地址来获取全部设备信息并初始化
pub fn init(dtb_pa: PhysicalAddress) {
    let dtb_va = VirtualAddress::from(dtb_pa);
    device_tree::init(dtb_va);
    println!("mod driver initialized")
}
