//! # OS by Rust - 一个用Rust编写的简单操作系统内核
//!
//! 这是一个教学用的操作系统内核，展示了如何使用Rust语言构建一个基本的操作系统。
//!
//! ## 主要特性
//! - 基本的内存管理（分页、堆分配）
//! - 中断处理（键盘、定时器）
//! - 异步任务系统
//! - VGA文本模式输出
//! - 串口通信

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)] // 用于 interrupt.rs
#![feature(const_mut_refs)]

/// 堆内存分配器
pub mod allocator;
/// 全局描述符表(GDT)和任务状态段(TSS)管理
pub mod gdt;
/// 输入子系统适配层
pub mod input;
/// 中断描述符表(IDT)和中断处理程序
pub mod interrupts;
/// 内存管理：分页、物理内存分配
pub mod memory;
/// 串口通信
pub mod serial;
/// 异步任务系统
pub mod task;
mod testing;
/// VGA文本缓冲区
pub mod vga_buffer;

extern crate alloc;

pub use testing::{exit_qemu, test_panic_handler, test_runner, QemuExitCode, Testable};
pub use input::reset_counters_for_test;

pub fn hlt_loop() -> ! {
    loop {
        // 让CPU在下一个中断触发之前休息一下
        x86_64::instructions::hlt();
    }
}

// 封装一个加载gdt和idt的函数
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    // x86_64 crate 中的 interrupts::enable 会执行特殊的 sti ("set interrupts") 指令来启用外部中断
    // 当试着执行 cargo run 后，double fault 异常几乎是立刻就被抛出了
    x86_64::instructions::interrupts::enable();
}

