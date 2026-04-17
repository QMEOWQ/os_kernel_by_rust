//! # 键盘输入异步处理
//!
//! 这个模块实现了键盘输入的异步处理，包括：
//! - 异步流接口
//! - 键盘事件解码和处理

use crate::{input, print};
use crate::task::executor;
use crate::task::timer;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::stream::{Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    // 使得该函数成为构造扫描码输入流的唯一方法
    pub fn new() -> Self {
        input::init_keyboard_input();
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        // 快速路径：如果队列中有数据，直接返回
        if let Some(scancode) = input::pop_keyboard_scancode() {
            return Poll::Ready(Some(scancode));
        }

        // 注册waker，然后再次检查队列以避免竞态条件
        input::register_keyboard_waker(cx.waker());

        // 再次检查队列，因为在注册waker之前可能有新的数据到达
        match input::pop_keyboard_scancode() {
            Some(scancode) => {
                // 如果找到了数据，清除waker并返回数据
                input::clear_keyboard_waker();
                Poll::Ready(Some(scancode))
            }
            None => {
                // 队列仍然为空，返回Pending等待唤醒
                Poll::Pending
            }
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        if !handle_debug_command(character) {
                            print!("{}", character);
                        }
                    }
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

fn handle_debug_command(command: char) -> bool {
    match command {
        's' | 'S' => {
            let stats = executor::global_stats_snapshot();
            crate::serial_println!(
                "[diag] tick={} active={} q_high={} q_normal={} wakers={} dropped_wakes={} dropped_scan={} uninit_scan={}",
                timer::current_tick(),
                stats.active_tasks,
                stats.queued_high_tasks,
                stats.queued_normal_tasks,
                stats.cached_wakers,
                stats.dropped_wakes,
                input::dropped_scancode_count(),
                input::uninitialized_scancode_count(),
            );
            true
        }
        'r' | 'R' => {
            input::reset_counters_for_test();
            crate::serial_println!("[diag] input counters reset");
            true
        }
        'h' | 'H' => {
            crate::serial_println!("[diag] commands: s=show stats, r=reset input counters, h=help");
            true
        }
        _ => false,
    }
}
