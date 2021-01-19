//! 预约和处理时钟中断
// 时钟中断也需要我们在初始化操作系统时开启, 我们同样只需使用 riscv 库中提供的接口即可。
use crate::sbi::set_timer;
use riscv::register::{time, sie};

// sstatus 寄存器中的 SIE 位决定中断是否能够打断 supervisor 线程
// 在这里我们需要允许时钟中断打断 内核态线程
// 另外，无论 SIE 位为什么值，中断都可以打断用户态的线程。


/// 初始化时钟中断
/// 
/// 开启时钟中断使能，并且预约第一次时钟中断
/// 我们会在线程开始运行时开启中断，而在操作系统初始化的过程中是不应该有中断的, 所以删去sie的设置
pub fn init() {
    unsafe {
        // 开启 STIE，允许时钟中断
        sie::set_stimer();
        // （删除）开启 SIE（不是 sie 寄存器），允许内核态被中断打断
        // sstatus::set_sie(); // 开启 sstatus 寄存器中的 SIE 位，与 sie 寄存器无关
    }
    // 设置下一次时钟中断
    set_next_timeout();
}

/// 时钟中断的间隔，单位是 CPU 指令
/// 越短的间隔可以让 CPU 调度资源更加细致，但同时也会导致更多资源浪费在操作系统上。
static INTERVAL: usize = 100000; // CPU 周期

/// 设置下一次时钟中断
/// 
/// 获取当前时间，加上中断间隔，通过 SBI 调用预约下一次中断
fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

/// 触发时钟中断计数
pub static mut TICKS: usize = 0;

/// 每一次时钟中断时调用
/// 由于没有一个接口来设置固定重复的时间中断间隔，因此我们需要在每一次时钟中断时，设置再下一次的时钟中断。
/// 设置下一次时钟中断，同时计数 +1
pub fn tick() {
    set_next_timeout();
    unsafe {
        TICKS += 1;
        // if TICKS % 100 == 0 {
        //     println!("{} tick", TICKS);
        // }
    }
}

