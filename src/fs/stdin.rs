//! 键盘输入 [`Stdin`]
// 输入流较为复杂：每当遇到系统调用时，通过中断或轮询方式获取字符：如果有，就进一步获取；如果没有就等待。直到收到约定长度的字符串才返回。
// 每一个键盘按键对于操作系统而言都是一次短暂的中断。
use super::*;
use alloc::collections::VecDeque;
// 输入流则需要配有一个缓冲区，我们可以用 alloc::collections::VecDeque 来实现
// 在遇到键盘中断时，调用 sbi_call 来获取字符并加入到缓冲区中。当遇到系统调用 sys_read 时，再相应从缓冲区中取出一定数量的字符。
// Q: 如果遇到了 sys_read 系统调用，而缓冲区并没有数据可以读取，应该如何让线程进行等待，而又不浪费 CPU 资源呢？

// A: 当一个线程调用 sys_read 而缓冲区为空时，就会将其加入条件变量的 watcher 中，同时在 Processor 中移出活跃线程。
// 而当键盘中断到来，读取到字符时，就会将线程重新放回调度器中，准备下一次调用。

lazy_static! {
    pub static ref STDIN: Arc<Stdin> = Default::default();
}

/// 控制台键盘输入，实现 [`INode`] 接口
#[derive(Default)]
pub struct Stdin {
    /// 从后插入，前段弹出
    buffer: Mutex<VecDeque<u8>>,
    /// 条件变量用于使等待输入的线程休眠
    condvar: Condvar,
}

impl INode for Stdin {
    /// Read bytes at `offset` into `buf`, return the number of bytes read.
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        if offset != 0 {
            // 不支持 offset
            Err(FsError::NotSupported)
        } else if self.buffer.lock().len() == 0 {
            // 缓冲区没有数据，将当前线程休眠
            self.condvar.wait();
            Ok(0)
        } else {
            let mut stdin_buffer = self.buffer.lock();
            for (i, byte) in buf.iter_mut().enumerate() {
                if let Some(b) = stdin_buffer.pop_front() {
                    *byte = b;
                } else {
                    return Ok(i);
                }
            }
            Ok(buf.len())
        }
    }

    /// Write bytes at `offset` from `buf`, return the number of bytes written.
    fn write_at(&self, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(FsError::NotSupported)
    }

    fn poll(&self) -> Result<PollStatus> {
        Err(FsError::NotSupported)
    }

    /// This is used to implement dynamics cast.
    /// Simply return self in the implement of the function.
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

impl Stdin {
    /// 向缓冲区插入一个字符，然后唤起一个线程
    pub fn push(&self, c: u8) {
        self.buffer.lock().push_back(c);
        self.condvar.notify_one();
    }
}
