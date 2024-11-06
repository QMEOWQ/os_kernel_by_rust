// 在Rust中，集成测试（integration tests）的约定是将其放到项目根目录中的 tests 目录下(即 src 的同级目录)。
// 无论是默认测试框架还是自定义测试框架都将自动获取并执行该目录下所有的测试。
// 所有的集成测试都是它们自己的可执行文件，并且与我们的 main.rs 完全独立。
// 这也就意味着每个测试都需要定义它们自己的函数入口点。
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_by_rust::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
    //loop {}
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
