
use riscv::register::{sstatus::Sstatus, scause::Scause};

// 中断处理过程中需要保存的上下文(Context)向量

// 在处理中断之前，必须要保存所有可能被修改的寄存器，并且在处理完成后恢复。
// 保存所有通用寄存器，sepc、scause 和 stval 这三个会被硬件自动写入的 CSR 寄存器，以及 sstatus (因为中断可能会涉及到权限的切换，以及中断的开关，这些都会修改 sstatus。)
// scause 以及 stval 将不会放在 Context 而仅仅被看做一个临时的变量

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,
    pub sepc: usize
}
