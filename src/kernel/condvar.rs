//! 条件变量
// 当一个线程调用 sys_read 而缓冲区为空时，就会将其加入条件变量的 watcher 中，同时在 Processor 中移出活跃线程。
// 而当键盘中断到来，读取到字符时，就会将线程重新放回调度器中，准备下一次调用。
use super::*;
use alloc::collections::VecDeque;

#[derive(Default)]
pub struct Condvar {
    /// 所有等待此条件变量的线程
    watchers: Mutex<VecDeque<Arc<Thread>>>,
}

impl Condvar {
    /// 令当前线程休眠，等待此条件变量
    pub fn wait(&self) {
        self.watchers
            .lock()
            .push_back(PROCESSOR.lock().current_thread());
        PROCESSOR.lock().sleep_current_thread();
    }

    /// 唤起一个等待此条件变量的线程
    pub fn notify_one(&self) {
        if let Some(thread) = self.watchers.lock().pop_front() {
            PROCESSOR.lock().wake_thread(thread);
        }
    }
}
