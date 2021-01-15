//! 实现控制台的字符 格式化输入和输出
//! 
//! # 格式化输出
//! 
//! [`core::fmt::Write`] trait 包含
//! - 需要实现的 [`write_str`] 方法
//! - 自带实现，但依赖于 [`write_str`] 的 [`write_fmt`] 方法
//! 
//! 我们声明一个类型，为其实现 [`write_str`] 方法后，就可以使用 [`write_fmt`] 来进行格式化输出
//! 
//! [`write_str`]: core::fmt::Write::write_str
//! [`write_fmt`]: core::fmt::Write::write_fmt

use crate::sbi::*; // 导入本项目的sbi.rs
use core::fmt::{self, Write};

/// 在屏幕上输出一个字符
/// 运行在 S 态的 OS 通过 ecall 发起 SBI 调用请求，RISC-V CPU 会从 S 态跳转到 M 态的 OpenSBI 固件，
/// OpenSBI 会检查 OS 发起的 SBI 调用的编号，如果编号在 0-8 之间，则进行处理，否则交由我们自己的中断处理程序处理
/// OpenSBI 提供了一些底层系统服务供我们在编写内核时使用, 是 S Mode 的 OS 和 M Mode 执行环境之间的标准接口约定。
/// 一般而言，a7(x17) 为 SBI 调用编号，a0(x10)、a1(x11) 和 a2(x12) 寄存器为 SBI 调用参数


/// 一个 [Zero-Sized Type]，实现 [`core::fmt::Write`] trait 来进行格式化输出
/// 
/// ZST 只可能有一个值（即为空），因此它本身就是一个单件
struct Stdout;

impl Write for Stdout {
    /// 打印一个字符串
    ///
    /// [`console_putchar`] sbi 调用每次接受一个 `usize`，但实际上会把它作为 `u8` 来打印字符。
    /// 因此，如果字符串中存在非 ASCII 字符，需要在 utf-8 编码下，对于每一个 `u8` 调用一次 [`console_putchar`]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buffer = [0u8; 4]; // 一个utf8字符可以转换为 4 byte (4 u8)
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                console_putchar(*code_point as usize); // 一个个输出即可，.iter()方法返回不可变引用
            }
        }
        Ok(()) // 返回Ok()
    }
}

/// 打印由 [`core::format_args!`] 格式化后的数据
/// 
/// [`print!`] 和 [`println!`] 宏都将展开成此函数
/// 
/// [`core::format_args!`]: https://doc.rust-lang.org/nightly/core/macro.format_args.html
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap(); // write_fmt 依赖于 write_str 的实现
}

/// 实现类似于标准库中的 `print!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => { // 一个字符串字面量 和 若干个参数
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// 实现类似于标准库中的 `println!` 宏
/// 
/// 使用实现了 [`core::fmt::Write`] trait 的 [`console::Stdout`]
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => { // 一个字符串字面量 和 若干个参数
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
