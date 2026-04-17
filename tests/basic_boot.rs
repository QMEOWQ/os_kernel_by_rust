#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os_by_rust::println;

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    os_by_rust::init();
    test_main();
    os_by_rust::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
