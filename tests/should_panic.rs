// 构造理应失败的测试, 一般用于验证传递无效参数时函数是否会失败
// 该集成测试可以从panic处理程序中返回一个成功错误代码
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_by_rust::serial_print;
use os_by_rust::{exit_qemu, serial_println, QemuExitCode};

// after set harness = false in Cargo.toml
// 通过 _start 函数来直接调用 should_fail 函数
#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("Testing should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     test_main();

//     loop{}
// }

// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//         serial_println!("[test did not panic]");
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     serial_println!("[ok]");
//     exit_qemu(QemuExitCode::Success);
//     loop {}
// }

// // cargo test --test should_panic
// // 该测试如我们预期的那样panic了。
// // 当我们将断言部分（即 assert_eq!(0, 1);）注释掉后，
// // 我们就会发现测试失败，并返回了 "test did not panic" 的信息。
// #[test_case]
// fn should_fail() {
//     serial_print!("Testing should_fail... ");
//     assert_eq!(0, 1);
// }
