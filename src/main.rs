//! # 全局属性
//! 
//! Step 0.1: 移除标准库依赖
// 项目默认是链接 Rust 标准库 std 的，它依赖于操作系统，因此我们需要显式通过 #![no_std] 将其禁用
// 但是println!这些东西是标准库的宏，所以此时会编译出错
//! - `#![no_std]`  
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`  
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口。告诉编译器我们不用常规的入口点
#![no_main]


// 还会提示缺失 panic_handler ，它默认使用标准库 std 中实现的函数并依赖于操作系统特殊的文件描述符
// 所以自己实现panic函数
// 类型为 PanicInfo 的参数包含了 panic 发生的文件名、代码行数和可选的错误信息。
// 这里我们用到了核心库 core，与标准库 std 不同，这个库不需要操作系统的支持
use core::panic::PanicInfo;
/// 当 panic 发生时会调用该函数，我们暂时将它的实现为一个死循环
/// 这个函数从不返回，所以他被标记为发散函数（Diverging Function）
/// 发散函数的返回类型称作 Never 类型（"never" type），记为 !
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// 报的第三个错误是:
// error: language item required, but not found: `eh_personality` (Exception Handling personality)
// 它是一个标记某函数用来实现 堆栈展开(Stack Unwinding) 处理功能的语义项
// 通常当程序出现了异常时，从异常点开始会沿着 caller 调用栈一层一层回溯，直到找到某个函数能够捕获这个异常或终止程序。这个过程称为堆栈展开。
// 当程序出现异常时，我们需要沿着调用栈一层层回溯上去回收每个 caller 中定义的局部变量（这里的回收包括 C++ 的 RAII 的析构以及 Rust 的 drop 等）避免造成捕获异常并恢复后的内存溢出。
// 简单起见，我们这里不会进一步捕获异常也不需要清理现场，我们设置为直接退出程序即可。


// Step 0.2: 移除运行时环境依赖
// 第四个错误: error: requires `start` lang_item
// 对于大多数语言，他们都使用了运行时系统（Runtime System），这可能导致 main 函数并不是实际执行的第一个函数。
// 以 Rust 语言为例，一个典型的链接了标准库的 Rust 程序会首先跳转到 C 语言运行时环境中的 crt0（C Runtime Zero）, 进入 C 语言运行时环境, 创建堆栈或设置寄存器参数 等
// C 语言运行时环境会跳转到 Rust 运行时环境的入口点（Entry Point）进入 Rust 运行时入口函数继续设置 Rust 运行环境
// 这个 Rust 的运行时入口点就是被 start 语义项标记的。
// Rust 运行时环境的入口点 start 结束之后才会调用 main 函数进入主程序。
// 所以我们需要重写覆盖整个 crt0 入口点

/// 覆盖 crt0 中的 _start 函数
/// 我们暂时将它的实现为一个死循环
#[no_mangle] // 告诉编译器对于此函数禁用编译期间的名称重整（Name Mangling）即确保编译器生成一个名为 _start 的函数，而非为了实现函数重载等而生成的形如 _ZN3blog_os4_start7hb173fedf945531caE 散列化后的函数名。
pub extern "C" fn _start() -> ! { // Rust 中的 FFI （Foreign Function Interface, 语言交互接口）语法, 表示此函数是一个 C 函数而非 Rust 函数
    loop {} // 由于程序会一直停在 crt0 的入口点，我们可以移除没用的 main 函数。
}

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

// fn main() {
//     //println!("Hello, rCore-Tutorial!");
// }
