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

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("Hello rCore-Tutorial!");
    panic!("end of rust_main")
}
