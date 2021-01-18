//! 驱动接口的定义
//!
//! 目前接口中只支持块设备类型
//! 
// 在写块设备驱动之前，我们先抽象驱动的概念，也方便后面网络设备等的介入。
use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::RwLock;

/// 驱动类型
///
/// 目前只有块设备，可能还有网络、GPU 设备等
#[derive(Debug, Eq, PartialEq)]
pub enum DeviceType {
    Block,
}

/// 驱动的接口
/// Driver 作为一个核心 trait 为上提供实现, 上层也就是 Driver 的使用侧（设备的抽象），而下层则是 Driver 的实现侧（设备的实现）
pub trait Driver: Send + Sync {
    /// 设备类型
    fn device_type(&self) -> DeviceType;

    /// 读取某个块到 buf 中（块设备接口）
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) -> bool {
        unimplemented!("not a block driver")
    }

    /// 将 buf 中的数据写入块中（块设备接口）
    fn write_block(&self, _block_id: usize, _buf: &[u8]) -> bool {
        unimplemented!("not a block driver")
    }
}

lazy_static! {
    /// 所有驱动
    pub static ref DRIVERS: RwLock<Vec<Arc<dyn Driver>>> = RwLock::new(Vec::new());
}
