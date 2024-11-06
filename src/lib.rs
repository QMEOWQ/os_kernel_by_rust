#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)] // 用于 interrupt.rs
#![feature(const_mut_refs)]

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer;
pub mod allocator;

extern crate alloc;

use core::panic::PanicInfo;
use x86_64::instructions::hlt;

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

// 实现自动添加打印语句
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
    //loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/*************
**** test ****
*************/

// 可以执行cargo test 运行所有测试
// 或执行cargo test --lib 来运行 lib.rs 及其子模块中包含的测试

// Entry point for `cargo test`
#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

// pub extern "C" fn _start() -> ! {
//     init();
//     test_main();
//     hlt_loop();
//     //loop {}
// }

// #[cfg(test)]
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     test_panic_handler(info)
// }
