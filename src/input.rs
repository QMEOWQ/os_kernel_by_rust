//! 输入子系统。
//!
//! 中断处理程序只依赖本模块，不直接依赖异步任务实现细节。
//! 键盘扫描码队列和唤醒机制由本模块统一管理。

use conquer_once::spin::OnceCell;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::Waker;
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;

#[cfg(all(feature = "input-drop-new", feature = "input-drop-old"))]
compile_error!("features `input-drop-new` and `input-drop-old` are mutually exclusive");

const KEYBOARD_SCANCODE_QUEUE_CAPACITY: usize = 100;

static KEYBOARD_SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static KEYBOARD_WAKER: AtomicWaker = AtomicWaker::new();
static DROPPED_SCANCODE_COUNT: AtomicU64 = AtomicU64::new(0);
static UNINITIALIZED_SCANCODE_COUNT: AtomicU64 = AtomicU64::new(0);

/// 初始化键盘输入队列。
pub fn init_keyboard_input() {
    KEYBOARD_SCANCODE_QUEUE
        .try_init_once(|| ArrayQueue::new(KEYBOARD_SCANCODE_QUEUE_CAPACITY))
        .expect("keyboard input queue should only be initialized once");
}

/// 将键盘扫描码推入输入管线。
pub fn push_keyboard_scancode(scancode: u8) {
    if let Ok(queue) = KEYBOARD_SCANCODE_QUEUE.try_get() {
        // 中断路径优先保证系统存活：所有策略都必须无panic。
        #[cfg(feature = "input-drop-old")]
        handle_scancode_with_drop_old(queue, scancode);
        #[cfg(feature = "input-drop-new")]
        handle_scancode_with_drop_new(queue, scancode);
        #[cfg(not(any(feature = "input-drop-new", feature = "input-drop-old")))]
        handle_scancode_with_drop_new(queue, scancode);
    } else {
        UNINITIALIZED_SCANCODE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

/// 弹出一个键盘扫描码。
pub fn pop_keyboard_scancode() -> Option<u8> {
    KEYBOARD_SCANCODE_QUEUE.try_get().ok().and_then(|queue| queue.pop())
}

/// 注册等待键盘输入的任务唤醒器。
pub fn register_keyboard_waker(waker: &Waker) {
    KEYBOARD_WAKER.register(waker);
}

/// 清理已注册的键盘输入唤醒器。
pub fn clear_keyboard_waker() {
    KEYBOARD_WAKER.take();
}

/// 已丢弃的键盘扫描码总数（队列满导致）。
pub fn dropped_scancode_count() -> u64 {
    DROPPED_SCANCODE_COUNT.load(Ordering::Relaxed)
}

/// 队列未初始化时接收到的扫描码总数。
pub fn uninitialized_scancode_count() -> u64 {
    UNINITIALIZED_SCANCODE_COUNT.load(Ordering::Relaxed)
}

/// 重置输入统计计数，仅用于测试场景。
pub fn reset_counters_for_test() {
    DROPPED_SCANCODE_COUNT.store(0, Ordering::Relaxed);
    UNINITIALIZED_SCANCODE_COUNT.store(0, Ordering::Relaxed);
}

fn handle_scancode_with_drop_new(queue: &ArrayQueue<u8>, scancode: u8) {
    if queue.push(scancode).is_ok() {
        KEYBOARD_WAKER.wake();
    } else {
        DROPPED_SCANCODE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "input-drop-old")]
fn handle_scancode_with_drop_old(queue: &ArrayQueue<u8>, scancode: u8) {
    if queue.push(scancode).is_ok() {
        KEYBOARD_WAKER.wake();
        return;
    }

    let _ = queue.pop();
    if queue.push(scancode).is_ok() {
        KEYBOARD_WAKER.wake();
    } else {
        DROPPED_SCANCODE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
