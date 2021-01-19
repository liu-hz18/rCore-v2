//! 线程 [`Thread`]

use super::*;
use core::hash::{Hash, Hasher};

/// 线程 ID 使用 `isize`，可以用负数表示错误
pub type ThreadID = isize;

/// 线程计数器，用于设置线程 ID
static mut THREAD_COUNTER: ThreadID = 0;

/// 线程的信息
pub struct Thread {
    /// 线程 ID
    pub id: ThreadID, // 用于唯一确认一个线程，它会在系统调用等时刻用到。
    /// 线程的栈
    pub stack: Range<VirtualAddress>, // 运行栈：每个线程都必须有一个独立的运行栈，保存运行时数据。这里只是记录栈的地址区间
    /// 所属的进程，使用 引用计数 增加安全性
    pub process: Arc<Process>, // 所属进程的记号：同一个进程中的多个线程，会共享页表、打开文件等信息。因此，我们将它们提取出来放到线程中。
    /// 用 `Mutex` 包装一些可变的变量
    pub inner: Mutex<ThreadInner>, // 因为线程一般使用 Arc<Thread> 来保存，它是不可变的，所以其中再用 Mutex 来包装一部分，让这部分可以修改。
}

/// 线程中需要可变的部分
pub struct ThreadInner {
    /// 线程执行上下文
    /// 当线程不在执行时，我们需要保存其上下文（其实就是一堆寄存器的值），这样之后才能够将其恢复
    /// 当且仅当线程被暂停执行时，`context` 为 `Some`
    pub context: Option<Context>,
    /// 是否进入休眠
    pub sleeping: bool,
    /// 是否已经结束
    pub dead: bool,
    /// priority, 用于Stride Scheduling 调度算法
    pub priority: usize,
}

// 单个线程级的操作
impl Thread {
    /// 准备执行一个线程
    /// 启动一个线程除了需要 Context，还需要切换页表
    /// 激活对应进程的页表，将 Context 放至内核栈顶，并返回其 Context
    /// 页表的切换不会影响OS运行，因为在中断期间是操作系统正在执行，而操作系统所用到的内核线性映射是存在于每个页表中的。
    /// 进一步说，每一个进程的 MemorySet 都会映射操作系统的空间，否则在遇到中断的时候，将无法执行异常处理。
    pub fn prepare(&self) -> *mut Context {
        // 激活页表，换入新线程的页表
        self.process.inner().memory_set.activate();
        // 取出 Context
        let parked_frame = self.inner().context.take().unwrap();
        // 将 Context 放至内核栈顶 (压栈)，之后会返回到 __restore 中，完成切换上下文并跳到线程入口
        unsafe { KERNEL_STACK.push_context(parked_frame) }
    }

    /// 发生时钟中断后暂停线程，保存当前线程的 `Context`
    pub fn park(&self, context: Context) {
        // 检查目前线程内的 context 应当为 None
        assert!(self.inner().context.is_none());
        // 将 Context 保存到线程中
        self.inner().context.replace(context);
    }

    /// 创建一个线程
    pub fn new(
        process: Arc<Process>, // 占用process所有权
        entry_point: usize,
        arguments: Option<&[usize]>,
        priority: usize,
    ) -> MemoryResult<Arc<Thread>> {
        // 让 所属进程 分配一段连续虚拟空间并映射一段物理空间，作为线程的栈
        // 也就是，线程时资源的使用者，该资源从进程那里获取，进程并不会使用这些资源，而只是向操作系统索取。
        // 页面段的权限包括: Flags::READABLE(R), Flags::WRITABLE(W). 以及 process 是否是用户态进程(U)
        let stack = process.alloc_page_range(STACK_SIZE, Flags::READABLE | Flags::WRITABLE)?;

        // 构建线程的 Context, 包括 sepc 设置为entry_point，sp设为stack.end.into()(即线程栈顶), 压入参数arguments(<8个), sstatus的spp位 = is_user 
        let context = Context::new(stack.end.into(), entry_point, arguments, process.is_user);

        // 打包成线程
        let thread = Arc::new(Thread {
            id: unsafe {
                THREAD_COUNTER += 1;
                THREAD_COUNTER
            },
            stack,   // 线程栈
            process, // 所属进程
            inner: Mutex::new(ThreadInner {
                context: Some(context), // 上下文
                sleeping: false, // 非休眠
                dead: false,     // 非kill
                priority: priority,
            }),
        });
        Ok(thread)
    }

    /// 上锁并获得可变部分 ThreadInner 的引用
    pub fn inner(&self) -> spin::MutexGuard<ThreadInner> {
        self.inner.lock()
    }

    /// fork
    /// fork 后应当为目前的线程复制一份几乎一样的拷贝，新线程与旧线程同属一个进程，公用页表和大部分内存空间，而新线程的栈是一份拷贝。
    pub fn fork(&self, current_context: Context) -> MemoryResult<Arc<Thread>> {
        // 让所属进程分配并映射一段空间，作为线程的栈
        let stack = self.process.alloc_page_range(STACK_SIZE, Flags::READABLE | Flags::WRITABLE)?;
        // 新线程的栈是原先线程栈的拷贝 (原样复制)
        for i in 0..STACK_SIZE {
            *VirtualAddress(stack.start.0 + i).deref::<u8>() = *VirtualAddress(self.stack.start.0 + i).deref::<u8>()
        }
        // 构建线程的 Context, 包括 sepc 设置为entry_point，sp设为stack.end.into()(即线程栈顶), 压入参数arguments(<8个), sstatus的spp位 = is_user 
        let mut context = current_context.clone();
        // sp 指向新线程的上下文
        context.set_sp( usize::from(stack.start) -  usize::from(self.stack.start) + current_context.sp() );
        // 打包成线程
        let thread = Arc::new(Thread {
            id: unsafe {
                THREAD_COUNTER += 1;
                THREAD_COUNTER
            },
            stack,   // 线程栈
            process: Arc::clone(&self.process), // 所属进程
            inner: Mutex::new(ThreadInner {
                context: Some(context), // 上下文
                sleeping: false, // 非休眠
                dead: false,     // 非kill
                priority: self.inner().priority.clone(),
            }),
        });
        Ok(thread)
    }
}

/// 通过线程 ID 来判等
impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// 通过线程 ID 来判等
///
/// 在 Rust 中，[`PartialEq`] trait 不要求任意对象 `a` 满足 `a == a`。
/// 将类型标注为 [`Eq`]，会沿用 `PartialEq` 中定义的 `eq()` 方法，
/// 同时声明对于任意对象 `a` 满足 `a == a`。
impl Eq for Thread {}

/// 通过线程 ID 来哈希
impl Hash for Thread {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_isize(self.id);
    }
}

/// 打印线程除了父进程以外的信息
impl core::fmt::Debug for Thread {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter
            .debug_struct("Thread")
            .field("thread_id", &self.id)
            .field("stack", &self.stack)
            .field("context", &self.inner().context)
            .finish()
    }
}
