use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::future::Future;
use std::pin::Pin;
use crossbeam_queue::ArrayQueue;
use futures::task::{self, ArcWake};

// 模拟OS中的TaskId
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl TaskId {
    pub fn new() -> TaskId {
        TaskId(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

// 模拟OS中的Task
pub struct Task {
    pub id: TaskId,
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}

// 模拟OS中的TaskWaker
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl ArcWake for TaskWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        if let Err(_) = arc_self.task_queue.push(arc_self.task_id) {
            println!("Task queue full, dropping task wake for {:?}", arc_self.task_id);
        }
    }
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        task::waker(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

// 模拟OS中的Executor
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same id already in tasks!");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    println!("Task {:?} completed", task_id);
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {
                    println!("Task {:?} pending", task_id);
                }
            }
        }
    }

    pub fn run_until_idle(&mut self) {
        while !self.task_queue.is_empty() {
            self.run_ready_tasks();
        }
    }
}

// 测试异步函数
async fn async_number() -> u32 {
    println!("Computing async number...");
    42
}

async fn example_task() {
    println!("Example task started");
    let num = async_number().await;
    println!("async number is: {}", num);
    println!("Example task completed");
}

async fn counter_task(name: &'static str, count: u32) {
    for i in 0..count {
        println!("{}: {}", name, i);
        // 模拟异步操作
        tokio::task::yield_now().await;
    }
    println!("{} finished", name);
}

#[tokio::main]
async fn main() {
    println!("=== Testing OS Async Task System ===\n");

    // 测试1: 基本的async/await功能
    println!("Test 1: Basic async/await");
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.run_until_idle();
    println!();

    // 测试2: 多个并发任务
    println!("Test 2: Multiple concurrent tasks");
    let mut executor = Executor::new();
    executor.spawn(Task::new(counter_task("Task A", 3)));
    executor.spawn(Task::new(counter_task("Task B", 3)));
    executor.spawn(Task::new(counter_task("Task C", 3)));
    executor.run_until_idle();
    println!();

    // 测试3: 嵌套异步调用
    println!("Test 3: Nested async calls");
    async fn nested_task() {
        println!("Nested task level 1");
        async fn level2() {
            println!("Nested task level 2");
            async fn level3() {
                println!("Nested task level 3");
            }
            level3().await;
        }
        level2().await;
        println!("Nested task completed");
    }

    let mut executor = Executor::new();
    executor.spawn(Task::new(nested_task()));
    executor.run_until_idle();
    println!();

    println!("=== All tests completed successfully! ===");
}
