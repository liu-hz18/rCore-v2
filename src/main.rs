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
//! - `#![feature(alloc_error_handler)]`
//!   我们使用了一个全局动态内存分配器，以实现原本标准库中的堆内存分配。
//!   而语言要求我们同时实现一个错误回调，这里我们直接 panic
#![feature(alloc_error_handler)]


#[macro_use]
mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;

extern crate alloc;

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

// sie 和 sip 寄存器分别保存不同中断种类的使能和触发记录
// RISC-V 中将中断分为三种：
// > 软件中断（Software Interrupt），对应 SSIE 和 SSIP
// > 时钟中断（Timer Interrupt），对应 STIE 和 STIP
// > 外部中断（External Interrupt），对应 SEIE 和 SEIP

// Step 2.1 动态内存分配
// 为了在我们的内核中支持动态内存分配，在 Rust 语言中，我们需要实现 Trait GlobalAlloc，将这个类实例化，并使用语义项 #[global_allocator] 进行标记。
// 这样的话，编译器就会知道如何使用我们提供的内存分配函数进行动态内存分配。
// 我们的需求是分配一块连续的、大小至少为 size 字节的虚拟内存，且对齐要求为 align 
// 在这里使用 Buddy System 来实现这件事情。

// Step 2.2 物理内存探测
// 通过 MMIO（Memory Mapped I/O）技术将外设映射到一段物理地址，这样我们访问其他外设就和访问物理内存一样了
// OpenSBI 固件 来完成对于包括物理内存在内的各外设的扫描，将扫描结果以 DTB（Device Tree Blob）的格式保存在物理内存中的某个地方。
// 随后 OpenSBI 固件会将其地址保存在 a1 寄存器中，给我们使用。
// [0x80000000, 0x88000000): DRAM, 128MB, 操作系统管理

// Step 2.3 物理内存管理，分配和回收
// 为了方便管理所有的物理页，我们需要实现一个分配器可以进行分配和回收的操作

// Step 3.1 虚拟内存Sv39, 三级页表
// 物理地址56位，虚拟地址64位. 虽然虚拟地址有 64 位，只有低 39 位有效。规定 63-39 位的值必须等于第 38 位的值，否则会认为该虚拟地址不合法，在访问时会产生异常。
// 物理页号PPN为 44 位，每个物理页大小为 4KB (PPO 12 bit)
// 虚拟页号VPN为 27 位，每个虚拟页大小为 4KB (VPO 12 bit)
// 虚拟地址到物理地址的映射以页为单位，也就是说把虚拟地址所在的虚拟页映射到一个物理页，然后再在这个物理页上根据页内偏移找到物理地址，从而完成映射。
// VPN2: [26:18](三级~1GB). VPN1: [17:9](二级~2MB). VPN0: [8:0](一级~4KB)
// 每级页表都用 9 位索引的，因此有 2^9 = 512 个页表项，而每个页表项都是 8 字节，因此每个页表大小都为 512 x 8 = 4KB, 正好是一个物理页的大小
// 我们可以将*二级页表项*的 R,W,X 设置为不是全 0 的，那么它将与一级页表项类似，只不过可以映射一个 2MB 的大页（Huge Page）.这样在 RISC-V 中，可以很方便地建立起大页机制。
// 但如果修改了 satp 寄存器，说明 OS 切换到了一个与先前映射方式完全不同的页表。此时快表里面存储的映射已经失效了，这种情况下 OS 要在修改 satp 的指令后面马上使用 sfence.vma 指令刷新整个 TLB。
// 我们手动修改一个页表项之后，也修改了映射，但 TLB 并不会自动刷新，我们也需要使用 sfence.vma 指令刷新 TLB
// 你可以在后面加上一个虚拟地址，这样 sfence.vma 只会刷新这个虚拟地址的映射。
// rCore中
// 内核代码: 虚拟地址空间中以 0xffffffff80200000 开头的一段高地址空间中, 线性平移

// Step 3.2 实现页表
// 加入了 页表数据结构 和 页表项数据结构

// Step 3.3 内核重映射
// 各个段之间的访问权限是不同的。在现在粗糙的映射下，我们甚至可以修改内核 .text 段的代码。因为我们通过一个标志位 W 为 1 的页表项完成映射。
// 我们考虑对这些段分别进行重映射，使得他们的访问权限被正确设置。
// 封装内存段 Segment
// 线性映射出现在内核空间中. 而为了支持每个用户进程看到的虚拟空间是一样的，我们不能全都用线性映射.

// Step 3.4 页面置换
// 当一个线程操作到那些不在物理内存中的虚拟地址时，就会产生缺页异常（Page Fault）。此时操作系统会介入，交换一部分物理内存和磁盘中的数据，使得需要访问的内存数据被放入物理内存之中。
// 在页表中，页表项的 Valid 位就表示对应的页面是否在物理内存中。因此，操作系统还必须更新页表，并刷新缓存。
// 传统 LRU (Least Recently Used) 算法。但这种算法需要维护一个优先队列，而且在每一次访问内存时都要更新。很显然这是不现实的，它带来的开销太大。

// Thinking: 假设某进程需要虚拟地址 A 到物理地址 B 的映射，这需要操作系统来完成。那么操作系统在建立映射时有没有访问 B？如果有，它是怎么在还没有映射的情况下访问 B 的呢？
// 建立映射不需要访问 B，而只需要操作页表即可。不过，通常程序都会需要操作系统建立映射的同时向页面中加载一些数据。此时，尽管 A→B 的映射尚不存在，因为我们将整个可用物理内存都建立了内核映射，所以操作系统仍然可以通过线性偏移量来访问到 B。

/// Rust 的入口函数
/// 在 entry.asm 中通过 jal 指令调用的，因此其执行完后会回到 entry.asm 中
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! { // 如果最后不是死循环或panic!，那么这个函数有返回值，所以就要去掉 -> !
    println!("Hello rCore-Tutorial!");
    // 初始化各种模块, 比如设置中断入口为 __interrupt, 以及开启时钟中断
    interrupt::init();
    memory::init();

    let remap = memory::mapping::MemorySet::new_kernel().unwrap();
    remap.activate();
    // 此时所有逻辑已经建立在了新构建的页表上，而不是那个粗糙的 boot_page_table 了
    // boot_page_table 并非没有用，它为我们构建重映射提供了支持，但终究我们会用更精细的页表和映射代替了它，实现了更细致的管理和安全性。
    println!("kernel remapped");

    panic!("end of rust_main")
}
