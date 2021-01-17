#![warn(unused_imports)]
use core::fmt;
use core::mem::zeroed;
use riscv::register::sstatus::{self, Sstatus, SPP::*};


// 中断处理过程中需要保存的上下文(Context)向量

// 在处理中断之前，必须要保存所有可能被修改的寄存器，并且在处理完成后恢复。
// 保存所有通用寄存器，sepc、scause 和 stval 这三个会被硬件自动写入的 CSR 寄存器，以及 sstatus (因为中断可能会涉及到权限的切换，以及中断的开关，这些都会修改 sstatus。)
// scause 以及 stval 将不会放在 Context 而仅仅被看做一个临时的变量
// 为了状态的保存与恢复，我们可以先用栈上的一小段空间来把需要保存的全部通用寄存器和 CSR 寄存器保存在栈上，保存完之后在跳转到 Rust 编写的中断处理函数

/// ### `#[repr(C)]` 属性
/// 要求 struct 按照 C 语言的规则进行内存分布，否则 Rust 可能按照其他规则进行内存排布
#[repr(C)] // 和C语言保持一致
#[derive(Clone, Copy)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,   // 具有许多状态位，控制全局中断使能等。
    pub sepc: usize         // Exception Program Counter, 用来记录触发中断的指令的地址。
}

/// 创建一个用 0 初始化的 Context
///
/// 这里使用 [`core::mem::zeroed()`] 来强行用全 0 初始化。
/// 因为在一些类型中，0 数值可能不合法（例如引用），所以 [`zeroed()`] 是 unsafe 的
impl Default for Context {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[allow(unused)]
impl Context {
    /// 获取栈指针
    pub fn sp(&self) -> usize {
        self.x[2]
    }

    /// 设置栈指针
    pub fn set_sp(&mut self, value: usize) -> &mut Self {
        self.x[2] = value;
        self
    }

    /// 获取返回地址
    pub fn ra(&self) -> usize {
        self.x[1]
    }

    /// 设置返回地址
    pub fn set_ra(&mut self, value: usize) -> &mut Self {
        self.x[1] = value;
        self
    }

    /// 按照函数调用规则写入参数
    ///
    /// 没有考虑一些特殊情况，例如超过 8 个参数，或 struct 空间展开
    pub fn set_arguments(&mut self, arguments: &[usize]) -> &mut Self {
        assert!(arguments.len() <= 8);
        self.x[10..(10 + arguments.len())].copy_from_slice(arguments);
        self
    }

    /// 为线程构建初始 `Context`
    pub fn new(
        stack_top: usize,
        entry_point: usize,
        arguments: Option<&[usize]>,
        is_user: bool,
    ) -> Self {
        let mut context = Self::default();

        // 设置栈顶指针
        context.set_sp(stack_top);
        // 设置初始参数
        if let Some(args) = arguments {
            context.set_arguments(args);
        }
        // 设置入口地址
        context.sepc = entry_point;

        // 设置 sstatus
        context.sstatus = sstatus::read();
        if is_user {
            context.sstatus.set_spp(User);
        } else {
            context.sstatus.set_spp(Supervisor);
        }
        // 这样设置 SPIE 位，使得替换 sstatus 后关闭中断，
        // 而在 sret 到用户线程时开启中断。详见 SPIE 和 SIE 的定义
        context.sstatus.set_spie(true);

        context
    }
}

const REG_NAMES: [&str; 32] = [
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
        )?;
        write!(f, "  sepc      = 0x{:016x}\n", self.sepc)?;
        for i in 0..32 {
            write!(f, "  {:9} = 0x{:016x}\n", REG_NAMES[i], self.x[i])?;
        }
        Ok(())
    }
}
