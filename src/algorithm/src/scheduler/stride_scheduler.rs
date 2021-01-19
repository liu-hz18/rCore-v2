//! Stride Scheduling 调度算法 [`StrideScheduler`]

use super::Scheduler;
use alloc::collections::LinkedList;

/// 将线程和调度信息打包
struct StrideThread<ThreadType: Clone + Eq> {
    /// ticket
    ticket: usize,
    /// stride
    pass: usize,
    /// 线程数据
    pub thread: ThreadType,
}

/// Stride Scheduling 调度算法
/// 为不同线程设置不同优先级，使得其获得与优先级成正比的运行时间。
pub struct StrideScheduler<ThreadType: Clone + Eq> {
    /// max stride
    big_stride:usize,
    current_min:usize,
    /// 带有调度信息的线程池
    pool: LinkedList<StrideThread<ThreadType>>,
}

/// `Default` 创建一个空的调度器
impl<ThreadType: Clone + Eq> Default for StrideScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            big_stride: 137,
            current_min: 0,
            pool: LinkedList::new(),
        }
    }
}

impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for StrideScheduler<ThreadType> {
    fn add_thread(&mut self, thread: ThreadType, priority: usize) {
            self.pool.push_back(StrideThread {
                ticket: priority,
                pass: self.current_min,
                thread,
            })
    }

    fn get_next(&mut self) -> Option<ThreadType> {
        // 计时
        if let Some(best) = self.pool.iter_mut().min_by(|x, y| {
            (x.pass)
                .cmp(&(y.pass))
        }) {
            if best.ticket == 0 {
                best.pass += self.big_stride;
            }else{
                best.pass += self.big_stride / ( best.ticket + 1 );
            }
            self.current_min = best.pass;
            Some(best.thread.clone())
        } else {
            None
        }
    }

    fn remove_thread(&mut self, thread: &ThreadType) {
        // 移除相应的线程并且确认恰移除一个线程
        let mut removed = self.pool.drain_filter(|t| t.thread == *thread);
        assert!(removed.next().is_some() && removed.next().is_none());
    }

    fn set_priority(&mut self, _thread: ThreadType, _priority: usize) {
        for x in self.pool.iter_mut(){
            if x.thread == _thread {
                x.ticket = _priority as usize;
            }
        }
    }
}
