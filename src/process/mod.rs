//! 管理进程 / 线程

mod config;
mod lock;
#[allow(clippy::module_inception)]
mod process;
mod processor;
mod thread;
mod kernel_stack;

use crate::interrupt::*;
use crate::memory::*;
use alloc::{sync::Arc};
use spin::Mutex;

pub use config::*;
pub use kernel_stack::KERNEL_STACK;
pub use lock::Lock;
pub use process::Process;
pub use processor::PROCESSOR;
pub use thread::Thread;
