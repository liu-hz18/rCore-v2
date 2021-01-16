#![warn(unused_imports)]
use core::fmt;
use riscv::register::{sstatus::Sstatus, scause::Scause};

// 中断处理过程中需要保存的上下文(Context)向量

// 在处理中断之前，必须要保存所有可能被修改的寄存器，并且在处理完成后恢复。
// 保存所有通用寄存器，sepc、scause 和 stval 这三个会被硬件自动写入的 CSR 寄存器，以及 sstatus (因为中断可能会涉及到权限的切换，以及中断的开关，这些都会修改 sstatus。)
// scause 以及 stval 将不会放在 Context 而仅仅被看做一个临时的变量
// 为了状态的保存与恢复，我们可以先用栈上的一小段空间来把需要保存的全部通用寄存器和 CSR 寄存器保存在栈上，保存完之后在跳转到 Rust 编写的中断处理函数


#[repr(C)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,   // 具有许多状态位，控制全局中断使能等。
    pub sepc: usize         // Exception Program Counter, 用来记录触发中断的指令的地址。
}

const reg_names: [&str; 32] = [
    "x0(zero)",
    "x1(ra)",
    "x2(sp)",
    "x3(gp)",
    "x4(tp)",
    "x5(t0)",
    "x6(t1)",
    "x7(t3)",
    "x8(fp/s0)",
    "x9(s1)",
    "x10(a0)",
    "x11(a1)",
    "x12(a2)",
    "x13(a3)",
    "x14(a4)",
    "x15(a5)",
    "x16(a6)",
    "x17(a7)",
    "x18(s2)",
    "x19(s3)",
    "x20(s4)",
    "x21(s5)",
    "x22(s6)",
    "x23(s7)",
    "x24(s8)",
    "x25(s9)",
    "x26(s10)",
    "x27(s11)",
    "x28(t3)",
    "x29(t4)",
    "x30(t5)",
    "x31(t6)",
];

// 加入Context的格式化输出
impl fmt::Debug for Context {
    // `f` 是一个缓冲区（buffer），此方法必须将格式化后的字符串写入其中
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  {:?} (uie={}, sie={}, upie={}, spie={})\n", 
            self.sstatus, self.sstatus.uie(), self.sstatus.sie(),
            self.sstatus.upie(), self.sstatus.spie()
        );
        write!(f, "  sepc      = 0x{:016x}\n", self.sepc);
        for i in 0..32 {
            write!(f, "  {:9} = 0x{:016x}\n", reg_names[i], self.x[i]);
        }
        Ok(())
    }
}
