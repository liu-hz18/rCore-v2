//! 文件系统
//!
//! 将读取第一个块设备作为根文件系统

use crate::drivers::{
    block::BlockDevice,
    driver::{DeviceType, DRIVERS},
};
//use crate::kernel::Condvar;
use alloc::{sync::Arc, vec::Vec};
use core::any::Any;
use lazy_static::lazy_static;
use rcore_fs_sfs::SimpleFileSystem;
use spin::Mutex;

mod config;
mod inode_ext;

pub use config::*;
pub use inode_ext::INodeExt;
pub use rcore_fs::{dev::block_cache::BlockCache, vfs::*};

// BlockCache
// 该模块也是 rcore-fs 提供的
// 提供了一个存储设备在内存 Cache 的抽象，通过调用 BlockCache::new(device, BLOCK_CACHE_CAPACITY) 就可以把 device 自动变为一个有 Cache 的设备。
// 根目录将会在我们第一次使用 ROOT_INODE 时进行初始化，而初始化的方式是找到全部设备驱动中的第一个存储设备作为根目录。

lazy_static! {
    /// 根文件系统的根目录的 INode
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        // 选择第一个块设备
        for driver in DRIVERS.read().iter() {
            if driver.device_type() == DeviceType::Block {
                let device = BlockDevice(driver.clone());
                // 动态分配一段内存空间作为设备 Cache
                let device_with_cache = Arc::new(BlockCache::new(device, BLOCK_CACHE_CAPACITY));
                // 最后我们用 SimpleFileSystem::open 打开并返回根节点即可。
                return SimpleFileSystem::open(device_with_cache)
                    .expect("failed to open SFS")
                    .root_inode();
            }
        }
        panic!("failed to load fs")
    };
}

/// 触发 [`static@ROOT_INODE`] 的初始化并打印根目录内容
pub fn init() {
    ROOT_INODE.ls();
    println!("mod fs initialized");
}


