//! 先入先出队列的调度器 [`FifoScheduler`]

use super::Scheduler;
use alloc::collections::LinkedList;

/// 采用 FIFO 算法的线程调度器
pub struct FifoScheduler<ThreadType: Clone + Eq> {
    pool: LinkedList<ThreadType>, // 单向链表，作为线程队列, ThreadType = Arc<Thread>
}

/// `Default` 创建一个空的调度器
impl<ThreadType: Clone + Eq> Default for FifoScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            pool: LinkedList::new(),
        }
    }
}

impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for FifoScheduler<ThreadType> {
    //type Priority = ();
    fn add_thread(&mut self, thread: ThreadType, _priority: usize) {
        // 加入链表尾部
        self.pool.push_back(thread);
    }
    fn get_next(&mut self) -> Option<ThreadType> {
        // 从头部取出放回尾部，同时将其返回
        if let Some(thread) = self.pool.pop_front() {
            self.pool.push_back(thread.clone());
            Some(thread)
        } else {
            None
        }
    }
    fn remove_thread(&mut self, thread: &ThreadType) {
        // 移除相应的线程 并且 确认恰移除一个线程
        let mut removed = self.pool.drain_filter(|t| t == thread); // O(n)查找
        assert!(removed.next().is_some() && removed.next().is_none());
    }
    // 优先级尚未实现
    fn set_priority(&mut self, _thread: ThreadType, _priority: usize) {}
}
