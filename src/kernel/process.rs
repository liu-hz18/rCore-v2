//! 进程相关的内核功能

use super::*;

pub(super) fn sys_exit(code: usize) -> SyscallResult {
    println!(
        "thread {} exit with code {}",
        PROCESSOR.lock().current_thread().id,
        code
    );
    SyscallResult::Kill
}

// 获得线程ID
pub(super) fn sys_get_tid() -> SyscallResult {
    SyscallResult::Proceed(PROCESSOR.lock().current_thread().id.clone())
}

// sys_fork 系统调用，使得该系统调用为父线程返回自身的线程 ID，而为子线程返回 0。
// fork 子进程
pub(super) fn sys_fork(context: &Context) -> SyscallResult {
    let id = PROCESSOR.lock().current_thread().id.clone();
    PROCESSOR.lock().fork_current_thread(context);
    if PROCESSOR.lock().current_thread().id.clone() == id {
        SyscallResult::Proceed(id)
    } else {
        SyscallResult::Proceed(0)
    }
}
