//! # 高效的任务执行器
//!
//! 这个执行器使用唤醒机制来避免不必要的轮询，提高性能。
//! 只有当任务被唤醒时才会重新调度执行。

use super::{Task, TaskId, TaskPriority};
use alloc::task::Wake;
use alloc::{collections::BTreeMap, sync::Arc};
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

const TASK_QUEUE_CAPACITY: usize = 100;
static DROPPED_WAKE_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_ACTIVE_TASKS: AtomicU64 = AtomicU64::new(0);
static LAST_QUEUED_HIGH_TASKS: AtomicU64 = AtomicU64::new(0);
static LAST_QUEUED_NORMAL_TASKS: AtomicU64 = AtomicU64::new(0);
static LAST_CACHED_WAKERS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    DuplicateTaskId,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorStats {
    pub active_tasks: usize,
    pub queued_high_tasks: usize,
    pub queued_normal_tasks: usize,
    pub cached_wakers: usize,
    pub dropped_wakes: u64,
}

pub fn global_stats_snapshot() -> ExecutorStats {
    ExecutorStats {
        active_tasks: LAST_ACTIVE_TASKS.load(Ordering::Relaxed) as usize,
        queued_high_tasks: LAST_QUEUED_HIGH_TASKS.load(Ordering::Relaxed) as usize,
        queued_normal_tasks: LAST_QUEUED_NORMAL_TASKS.load(Ordering::Relaxed) as usize,
        cached_wakers: LAST_CACHED_WAKERS.load(Ordering::Relaxed) as usize,
        dropped_wakes: DROPPED_WAKE_COUNT.load(Ordering::Relaxed),
    }
}

/// 基于唤醒机制的高效任务执行器
///
/// 这个执行器维护一个任务队列，只有被唤醒的任务才会被重新调度。
/// 当没有任务需要执行时，CPU会进入休眠状态以节省电力。
pub struct Executor {
    /// 存储所有任务的映射表
    tasks: BTreeMap<TaskId, Task>,
    /// 高优先级待执行任务队列
    high_priority_task_queue: Arc<ArrayQueue<TaskId>>,
    /// 普通优先级待执行任务队列
    normal_priority_task_queue: Arc<ArrayQueue<TaskId>>,
    /// 缓存每个任务的唤醒器
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    /// 创建一个新的执行器
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            high_priority_task_queue: Arc::new(ArrayQueue::new(TASK_QUEUE_CAPACITY)),
            normal_priority_task_queue: Arc::new(ArrayQueue::new(TASK_QUEUE_CAPACITY)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// 添加一个新任务到执行器
    ///
    /// # Panics
    /// 如果任务ID已经存在或任务队列已满，会panic
    pub fn spawn(&mut self, task: Task) {
        self.try_spawn(task)
            .expect("failed to spawn task into executor");
    }

    /// 尝试添加一个新任务到执行器，避免在容量压力下panic。
    pub fn try_spawn(&mut self, task: Task) -> Result<(), SpawnError> {
        let task_id = task.id;
        let task_priority = task.priority;
        if self.tasks.insert(task_id, task).is_some() {
            return Err(SpawnError::DuplicateTaskId);
        }

        let queue_push_result = match task_priority {
            TaskPriority::High => self.high_priority_task_queue.push(task_id),
            TaskPriority::Normal => self.normal_priority_task_queue.push(task_id),
        };

        if queue_push_result.is_err() {
            self.tasks.remove(&task_id);
            return Err(SpawnError::QueueFull);
        }

        Ok(())
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            high_priority_task_queue,
            normal_priority_task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) =
            pop_next_task_id(high_priority_task_queue, normal_priority_task_queue)
        {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let task_priority = task.priority;

            let high_queue = high_priority_task_queue.clone();
            let normal_queue = normal_priority_task_queue.clone();
            let waker = waker_cache.entry(task_id).or_insert_with(|| {
                TaskWaker::new(task_id, task_priority, high_queue, normal_queue)
            });

            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.update_stats_snapshot();
            self.sleep_if_idle();
        }
    }

    pub fn stats(&self) -> ExecutorStats {
        ExecutorStats {
            active_tasks: self.tasks.len(),
            queued_high_tasks: self.high_priority_task_queue.len(),
            queued_normal_tasks: self.normal_priority_task_queue.len(),
            cached_wakers: self.waker_cache.len(),
            dropped_wakes: DROPPED_WAKE_COUNT.load(Ordering::Relaxed),
        }
    }

    fn update_stats_snapshot(&self) {
        LAST_ACTIVE_TASKS.store(self.tasks.len() as u64, Ordering::Relaxed);
        LAST_QUEUED_HIGH_TASKS.store(self.high_priority_task_queue.len() as u64, Ordering::Relaxed);
        LAST_QUEUED_NORMAL_TASKS
            .store(self.normal_priority_task_queue.len() as u64, Ordering::Relaxed);
        LAST_CACHED_WAKERS.store(self.waker_cache.len() as u64, Ordering::Relaxed);
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.high_priority_task_queue.is_empty() && self.normal_priority_task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }

    // fn sleep_if_idle(&self) {
    //     if self.task_queue.is_empty() {
    //         // <--- interrupt can happen here
    //         x86_64::instructions::hlt();
    //     }
    // }
}

// TaskWaker is a waker that sends the task_id to the task_queue when woken.
struct TaskWaker {
    task_id: TaskId,
    task_priority: TaskPriority,
    high_priority_task_queue: Arc<ArrayQueue<TaskId>>,
    normal_priority_task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(
        task_id: TaskId,
        task_priority: TaskPriority,
        high_priority_task_queue: Arc<ArrayQueue<TaskId>>,
        normal_priority_task_queue: Arc<ArrayQueue<TaskId>>,
    ) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_priority,
            high_priority_task_queue,
            normal_priority_task_queue,
        }))
    }

    fn wake_task(&self) {
        // 任务已经在队列中时，重复唤醒可安全丢弃，避免队列满导致panic。
        let push_result = match self.task_priority {
            TaskPriority::High => self.high_priority_task_queue.push(self.task_id),
            TaskPriority::Normal => self.normal_priority_task_queue.push(self.task_id),
        };
        if push_result.is_err() {
            DROPPED_WAKE_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn pop_next_task_id(
    high_priority_task_queue: &Arc<ArrayQueue<TaskId>>,
    normal_priority_task_queue: &Arc<ArrayQueue<TaskId>>,
) -> Option<TaskId> {
    high_priority_task_queue
        .pop()
        .or_else(|| normal_priority_task_queue.pop())
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
