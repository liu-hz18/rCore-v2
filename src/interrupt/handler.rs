// 为了让硬件能够找到我们编写的 __interrupt 入口，在操作系统初始化时，需要将其写入 stvec 寄存器中
use super::context::Context;
use super::timer;
use crate::process::PROCESSOR;
use riscv::register::{
    stvec, sie,
    scause::{Exception, Interrupt, Scause, Trap}
};

global_asm!(include_str!("./interrupt.asm"));

/// 初始化中断处理
///
/// 把中断入口 `__interrupt` 写入 `stvec` 中，并且开启中断使能
pub fn init() {
    unsafe {
        extern "C" {
            /// `interrupt.asm` 中的中断入口
            fn __interrupt();
        }
        // 使用 Direct 模式，将中断入口设置为 `__interrupt`
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
        // 开启外部中断使能
        sie::set_sext();

    }
}

/// 中断的处理入口
/// 
/// `interrupt.asm` 首先保存寄存器至 Context，其作为参数和 scause 以及 stval 一并传入此函数
/// 参数的传入是通过汇编实现的，在 /interrupt.asm 中，占用a0, a1, a2寄存器，其中a0是个指针(sp), 对应&mut Context
/// 具体的中断类型需要根据 scause 来推断，然后分别处理
/// 为了实现线程的切换，我们让 handle_interrupt 返回一个 *mut Context
/// 如果需要切换线程，就将前一个线程的 Context 保存起来换上新的线程的 Context
/// 如果不需要切换，那么直接返回原本的 Context 即可。
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    // 首先检查线程是否已经结束（内核线程会自己设置标记来结束自己）
    {
        let mut processor = PROCESSOR.lock();
        let current_thread = processor.current_thread();
        if current_thread.as_ref().inner().dead { // 如果已经结束，就执行退出操作。
            println!("thread {} exit", current_thread.id);
            processor.kill_current_thread(); // 处理机将其移出调度序列
            return processor.prepare_next_thread(); // 准备下一个线程，参与调度，返回下一个线程的上下文*mut Context，编译器会负责把它放在a0寄存器中
        }
    }
    // 可以通过 Debug 来查看发生了什么中断
    // println!("{:x?}", scause.cause());
    // 根据中断类型来处理，返回的 Context 必须位于放在内核栈顶
    match scause.cause() {
        // 断点中断（ebreak）
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        // Load Fault, 访问不存在地址
        Trap::Exception(Exception::LoadFault) => loadfault(context, stval),
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        
        // 其他情况，终止当前线程
        _ => fault(context, scause, stval),
    }
    // panic!("Interrupted: {:?}", scause.cause()); // panic之后就退出了，没有返回
}

/// 处理 ebreak 断点
/// 
/// 继续执行，其中 `sepc` 增加 2 字节，以跳过当前这条 `ebreak` 指令
fn breakpoint(context: &mut Context) -> *mut Context {
    println!("Breakpoint at 0x{:016x}", context.sepc);
    context.sepc += 2;
    context // 当发生断点中断时，直接返回原来的上下文（修改一下 sepc）
}

/// 处理时钟中断
/// 时钟中断时切换线程
/// 目前只会在 [`timer`] 模块中进行计数，同时设置下一次时钟中断
fn supervisor_timer(context: &Context) -> *mut Context {
    timer::tick();
    PROCESSOR.lock().park_current_thread(context); // 时钟中断时切换线程，完成一次线程调度
    PROCESSOR.lock().prepare_next_thread()
}

/// 处理LoadFault
/// 
/// 直接panic, 终止程序
fn loadfault(context: &Context, stval: usize) -> *mut Context {
    if stval == 0x0_usize { // 如果程序想要非法访问的地址是 0x0，则打印 SUCCESS!
        println!("SUCCESS!");
    }
    println!("LoadFault: \n{:?}\n  stval = 0x{:016x}", context, stval);
    PROCESSOR.lock().kill_current_thread(); // 无法处理，杀死当前线程
    // 跳转到 PROCESSOR 调度的下一个线程
    PROCESSOR.lock().prepare_next_thread()
}

/// 出现未能解决的异常
fn fault(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    println!(
        "Unresolved interrupt: {:?}\n{:x?}\n  stval = 0x{:016x}",
        scause.cause(),
        context,
        stval
    );
    PROCESSOR.lock().kill_current_thread(); // 无法处理，杀死当前线程
    // 跳转到 PROCESSOR 调度的下一个线程
    PROCESSOR.lock().prepare_next_thread()
}
