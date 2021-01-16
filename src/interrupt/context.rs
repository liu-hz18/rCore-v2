#![warn(unused_imports)]
use riscv::register::{sstatus::Sstatus, scause::Scause};

// 中断处理过程中需要保存的上下文(Context)向量

// 在处理中断之前，必须要保存所有可能被修改的寄存器，并且在处理完成后恢复。
// 保存所有通用寄存器，sepc、scause 和 stval 这三个会被硬件自动写入的 CSR 寄存器，以及 sstatus (因为中断可能会涉及到权限的切换，以及中断的开关，这些都会修改 sstatus。)
// scause 以及 stval 将不会放在 Context 而仅仅被看做一个临时的变量
// 为了状态的保存与恢复，我们可以先用栈上的一小段空间来把需要保存的全部通用寄存器和 CSR 寄存器保存在栈上，保存完之后在跳转到 Rust 编写的中断处理函数


#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,   // 具有许多状态位，控制全局中断使能等。
    pub sepc: usize         // Exception Program Counter, 用来记录触发中断的指令的地址。
}
