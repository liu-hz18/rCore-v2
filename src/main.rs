//! # 全局属性
//! 
//! Step 0.1: 移除标准库依赖
// 项目默认是链接 Rust 标准库 std 的，它依赖于操作系统，因此我们需要显式通过 #![no_std] 将其禁用
// 但是println!这些东西是标准库的宏，所以此时会编译出错
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口。告诉编译器我们不用常规的入口点
#![no_main]
//! - `#![feature(global_asm)]`  
//!   内嵌整个汇编文件
#![feature(global_asm)]
//! # 一些 unstable 的功能需要在 crate 层级声明后才可以使用
//! - `#![feature(llvm_asm)]`  
//!   内嵌汇编
#![feature(llvm_asm)]
//! - `#![feature(panic_info_message)]`  
//!   panic! 时，获取其中的信息并打印
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod panic;
mod sbi;


// 汇编编写的程序入口，具体见该文件 entry.asm
global_asm!(include_str!("entry.asm"));

// Step 0.2: 移除运行时环境依赖
// 第四个错误: error: requires `start` lang_item
// 对于大多数语言，他们都使用了运行时系统（Runtime System），这可能导致 main 函数并不是实际执行的第一个函数。
// 以 Rust 语言为例，一个典型的链接了标准库的 Rust 程序会首先跳转到 C 语言运行时环境中的 crt0（C Runtime Zero）, 进入 C 语言运行时环境, 创建堆栈或设置寄存器参数 等
// C 语言运行时环境会跳转到 Rust 运行时环境的入口点（Entry Point）进入 Rust 运行时入口函数继续设置 Rust 运行环境
// 这个 Rust 的运行时入口点就是被 start 语义项标记的。
// Rust 运行时环境的入口点 start 结束之后才会调用 main 函数进入主程序。
// 所以我们需要重写覆盖整个 crt0 入口点

// Step 0.3 编译为裸机目标
// 此时会报链接错误，因为：链接器的默认配置假定程序依赖于 C 语言的运行时环境，但我们的程序并不依赖于它。
// 为了解决这个错误，我们需要告诉链接器，它不应该包含 C 语言运行时环境。
// 在这里，我们选择编译为裸机目标（Bare Metal Target），不链接任何运行时环境
// 为了描述不同的环境，Rust 使用一个称为目标三元组（Target Triple）的字符串 <arch><sub>-<vendor>-<sys>-<abi>
// x86_64-unknown-linux-gnu:  CPU 架构 x86_64, 供应商 unknown, 操作系统 linux, 二进制接口 gnu

// 我们可以另选一个底层没有操作系统的运行环境
// $ rustup target add riscv64imac-unknown-none-elf
// $ cargo build --target riscv64imac-unknown-none-elf
// 产物在 os/target/riscv64imac-unknown-none-elf/debug/os

// $ file target/riscv64imac-unknown-none-elf/debug/os 查看文件信息
// $ rust-objdump target/riscv64imac-unknown-none-elf/debug/os -x --arch-name=riscv64 查看程序元信息
// for more information, see https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-0/guide/part-5.html
// $ rust-objdump target/riscv64imac-unknown-none-elf/debug/os -d --arch-name=riscv64 查看反汇编信息

// 从 elf 格式可执行文件生成内核镜像
// $ rust-objcopy target/riscv64imac-unknown-none-elf/debug/os --strip-all -O binary target/riscv64imac-unknown-none-elf/debug/kernel.bin
// --strip-all 表明丢弃所有符号表及调试信息，-O binary 表示输出为二进制文件

// Step 0.4 调整内存布局, 改变它的链接地址
// 对于 OS 内核，一般都将其地址空间放在高地址上。
// 并且在 QEMU 模拟的 RISC-V 中，DRAM 内存的物理地址是从 0x80000000 开始，有 128MB 大小

// Step 0.5 重写程序入口点
// 在基于 RISC-V 的计算机系统中，OpenSBI (bootloader) 是一种固件。
// OpenSBI 固件运行在特权级别很高的计算机硬件环境中，即 RISC-V 64 的 M Mode（CPU 加电后也就运行在 M Mode）
// 我们将要实现的 OS 内核运行在 S Mode, 支持现代类 Unix 操作系统所需要的 基于页面的虚拟内存机制 是其核心。

// Step 0.6 运行QEMU
// $ qemu-system-riscv64 --machine virt --nographic --bios default
// QEMU 可以使用 ctrl+a 再按下 x 键退出。

// Step 1.1 中断
// 中断是我们在操作系统上首先实现的功能，因为它是操作系统所有功能的基础。
// 默认所有中断实际上是交给机器态处理的，但是为了实现更多功能，机器态会将某些中断交由内核态处理。这些异常也正是我们编写操作系统所需要实现的。
// 机器态可以通过异常委托机制（Machine Interrupt Delegation）将一部分中断设置为不经过机器态，直接由内核态处理

// 发生中断时，硬件自动填写的寄存器:
// sepc:  Exception Program Counter, 用来记录触发中断的指令的地址。
// scause: 记录中断是否是硬件中断，以及具体的中断原因。
// stval: scause 不足以存下中断所有的必须信息。例如缺页异常，就会将 stval 设置成需要访问但是不在内存中的地址，以便于操作系统将这个地址所在的页面加载进来。
// 指导硬件处理中断的寄存器:
// stvec: 设置内核态中断处理流程的入口地址。存储了一个基址 BASE 和模式 MODE (MODE 为 0 表示 Direct 模式，即遇到中断便跳转至 BASE 进行执行。MODE 为 1 表示 Vectored 模式，此时 BASE 应当指向一个向量，存有不同处理流程的地址，遇到中断会跳转至 BASE + 4 * cause 进行处理流程。)
// sstatus: 具有许多状态位，控制全局中断使能等。
// sie: Supervisor Interrupt Enable. 用来控制具体类型中断的使能，例如其中的 STIE 控制时钟中断使能。
// sip: Supervisor Interrupt Pending. 记录每种中断是否被触发。仅当 sie 和 sip 的对应位都为 1 时，意味着开中断且已发生中断，这时中断最终触发。
// sscratch: 在用户态，sscratch 保存内核栈的地址；在内核态，sscratch 的值为 0。为了能够执行内核态的中断处理流程，仅有一个入口地址是不够的。中断处理流程很可能需要使用栈，而程序当前的用户栈是不安全的。因此，我们还需要一个预设的安全的栈空间，存放在这里。可以在遇到中断时通过 sscratch 中的值判断中断前程序是否处于内核态。

// 中断指令
// ecall: 触发中断，进入更高一层的中断处理流程之中。用户态进行系统调用进入内核态中断处理流程，内核态进行 SBI 调用进入机器态中断处理流程，使用的都是这条指令。
// sret: 从内核态返回用户态，同时将 pc 的值设置为 sepc（如果需要返回到 sepc 后一条指令，就需要在 sret 之前修改 sepc 的值）
// ebreak: 触发一个断点。
// mret: 从机器态返回内核态，同时将 pc 的值设置为 mepc。

// 在处理中断之前，必须要保存所有可能被修改的寄存器，并且在处理完成后恢复。
// 保存所有通用寄存器，sepc、scause 和 stval 这三个会被硬件自动写入的 CSR 寄存器，以及 sstatus (因为中断可能会涉及到权限的切换，以及中断的开关，这些都会修改 sstatus。)
// scause 以及 stval 将不会放在 Context 而仅仅被看做一个临时的变量

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rCore-Tutorial!");
    panic!("end of rust_main")
}
