//! 内核栈 [`KernelStack`]
//!
//! 用户态的线程出现中断时，因为用户栈无法保证可用性，中断处理流程必须在内核栈上进行。
//! 所以我们创建一个公用的内核栈，即当发生中断时，会将 Context 写到内核栈顶。
//!
//! ### 线程 [`Context`] 的存放
//! > 1. 线程初始化时，一个 `Context` 放置在内核栈顶，`sp` 指向 `Context` 的位置
//! >   （即 栈顶 - `size_of::<Context>()`）
//! > 2. 切换到线程，执行 `__restore` 时，将 `Context` 的数据恢复到寄存器中后，
//! >   会将 `Context` 出栈（即 `sp += size_of::<Context>()`），
//! >   然后保存 `sp` 至 `sscratch`（此时 `sscratch` 即为内核栈顶）
//! > 3. 发生中断时，将 `sscratch` 和 `sp` 互换，入栈一个 `Context` 并保存数据
//!
//! 容易发现，线程的 `Context` 一定保存在内核栈顶。因此，当线程需要运行(切换到)时，
//! 从 [`Thread`] 中取出 `Context` 然后置于内核栈顶即可

// 做法:
// 1. 预留一段空间作为内核栈
// 2. 运行线程时，在 sscratch 寄存器中保存内核栈顶指针
// 3. 如果线程遇到中断，则将 Context 压入 sscratch 指向的栈中（Context 的地址为 sscratch - size_of::<Context>()），同时用新的栈地址来替换 sp（此时 sp 也会被复制到 a0 作为 handle_interrupt 的参数）
// 4. 从中断中返回时（__restore 时），a0 应指向被压在内核栈中的 Context。此时出栈 Context 并且将栈顶保存到 sscratch 中

use super::*;
use core::mem::size_of;

/// 内核栈
#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

/// 公用的内核栈
pub static mut KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

// 创建线程时，需要使用的操作就是在内核栈顶压入一个初始状态 Context
impl KernelStack {
    /// 在栈顶加入 Context 并且返回新的栈顶指针
    pub fn push_context(&mut self, context: Context) -> *mut Context {
        // 栈顶sp
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
        // Context 的位置
        let push_address = (stack_top - size_of::<Context>()) as *mut Context; // 编译器负责解析这个指针
        // Context 压入栈顶
        unsafe {
            *push_address = context; // 编译器负责对内存对应位置依次赋值
        }
        push_address
    }
}

// 关于内核栈(和sscratch)的作用的探讨:
/*
    如果不使用 sscratch 提供内核栈，而是像原来一样，遇到中断就直接将上下文压栈, 会有什么问题?
    1. 一种情况不会出现问题： 只运行一个非常善意的线程，比如 loop {}
    2. 一种情况导致异常无法处理（指无法进入 handle_interrupt）:
        线程把自己的 sp 搞丢了，比如 mv sp, x0。此时无法保存寄存器，也没有能够支持操作系统正常运行的栈
    3. 一种情况导致产生嵌套异常（指第二个异常能够进行到调用 handle_interrupt 时，不考虑后续执行情况）
        运行两个线程。在两个线程切换的时候，会需要切换页表。但是此时操作系统运行在前一个线程的栈上，一旦切换，再访问栈就会导致缺页，因为每个线程的栈只在自己的页表中
    4. 一种情况导致一个用户进程（先不考虑是怎么来的）可以将自己变为内核进程，或以内核态执行自己的代码
        用户进程巧妙地设计 sp，使得它恰好落在内核的某些变量附近，于是在保存寄存器时就修改了变量的值。这相当于任意修改操作系统的控制信息
*/
