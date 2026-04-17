//! # 异步任务系统
//!
//! 这个模块实现了一个基本的异步任务系统，包括：
//! - 任务抽象和唯一ID生成
//! - 简单的轮询执行器
//! - 高效的唤醒机制执行器
//! - 键盘输入的异步处理

use alloc::boxed::Box;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

/// 高效的任务执行器（基于唤醒机制）
pub mod executor;
/// 键盘输入异步处理
pub mod keyboard;
/// 简单的任务执行器（轮询所有任务）
pub mod simple_executor;
/// 基于时钟tick的定时/休眠能力
pub mod timer;

pub use timer::sleep_ticks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    High,
    Normal,
}

/// 异步任务的抽象
///
/// 每个任务都有一个唯一的ID和一个Future，可以被执行器调度执行
pub struct Task {
    pub id: TaskId,
    pub priority: TaskPriority,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// 创建一个新的任务
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Self::new_with_priority(future, TaskPriority::Normal)
    }

    pub fn new_with_priority(
        future: impl Future<Output = ()> + 'static,
        priority: TaskPriority,
    ) -> Task {
        Task {
            id: TaskId::new(),
            priority,
            future: Box::pin(future),
        }
    }

    /// 允许执行器轮询存储的future
    pub(crate) fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}

/// 任务的唯一标识符
///
/// 使用原子计数器生成，确保每个任务都有唯一的ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    /// 生成一个新的唯一任务ID
    pub fn new() -> TaskId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
