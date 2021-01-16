//! 中断模块
//! 
//! 

mod handler;
mod context;

/// 初始化中断相关的子模块, 简单封装 handler::init
/// 
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    println!("mod interrupt initialized.");
}
