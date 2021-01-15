//! 调用 Machine 层的操作
// 目前还不会用到全部的 SBI 调用，暂时允许未使用的变量或函数
#![allow(unused)]
// 一般而言，a7(x17) 为 SBI 调用编号
// 如果编号在 0-8 之间，则OpenSBI进行处理，否则交由我们自己的中断处理程序处理
const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

// 对于参数比较少且是基本数据类型的时候，我们从左到右使用寄存器 a0 到 a7 就可以完成参数的传递。
// 前三个参数分别代表接口可能所需的三个输入参数，最后一个 which 用来区分我们调用的是哪个接口（SBI Extension ID）
/// SBI 调用
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret) // 前面的 = 表明汇编代码会修改该寄存器x10(a0)并作为最后的返回值。
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which) // 分别通过寄存器 x10、x11、x12 和 x17（这四个寄存器又名 a0、a1、a2 和 a7） 传入参数 arg0、arg1、arg2 和 which
            : "memory"      // 如果汇编可能改变内存，则需要加入 memory 选项
            : "volatile");  // 防止编译器做激进的优化（如调换指令顺序等破坏 SBI 调用行为的优化）
    }
    ret
}

/// 向控制台输出一个字符
///
/// 需要注意我们不能直接使用 Rust 中的 char 类型
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// 从控制台中读取一个字符
///
/// 没有读取到字符则返回 -1
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

/// 调用 SBI_SHUTDOWN 来关闭操作系统（直接退出 QEMU）
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    unreachable!()
}
