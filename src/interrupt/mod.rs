//! 中断模块
//! 
//! 
mod handler;
mod context;
mod timer;

/// 初始化中断相关的子模块, 简单封装 一些 init
/// 
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init(); // 把中断入口 `__interrupt` 写入 `stvec` 中，并且开启中断使能
    timer::init(); // 开启时钟中断使能，并且预约第一次时钟中断
    println!("mod interrupt initialized.");
}
