//! 代替 std 库，实现 panic 和 abort 的功能

// 还会提示缺失 panic_handler ，它默认使用标准库 std 中实现的函数并依赖于操作系统特殊的文件描述符
// 所以自己实现panic函数
// 类型为 PanicInfo 的参数包含了 panic 发生的文件名、代码行数和可选的错误信息。
// 这里我们用到了核心库 core，与标准库 std 不同，这个库不需要操作系统的支持
use core::panic::PanicInfo;
use crate::sbi::shutdown; // 导入本项目的sbi.rs的shutdown函数

/// 打印 panic 的信息并 [`shutdown`]
///
/// ### `#[panic_handler]` 属性
/// 声明此函数是 panic 的回调
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // `\x1b[??m` 是控制终端字符输出格式的指令，在支持的平台上可以改变文字颜色等等，这里使用红色
    // 参考：https://misc.flogisoft.com/bash/tip_colors_and_formatting
    //
    // 需要全局开启 feature(panic_info_message) 才可以调用 .message() 函数
    println!("\x1b[1;31mpanic: '{}'\x1b[0m", info.message().unwrap());
    shutdown()
}

/// 终止程序
/// 
/// 调用 [`panic_handler`]
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
