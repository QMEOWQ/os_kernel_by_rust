#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    test_main();
    os_by_rust::exit_qemu(os_by_rust::QemuExitCode::Success);
    os_by_rust::hlt_loop();
}

#[test_case]
fn input_queue_lifecycle_and_counters() {
    os_by_rust::input::reset_counters_for_test();

    os_by_rust::input::push_keyboard_scancode(0x1e);
    assert_eq!(os_by_rust::input::uninitialized_scancode_count(), 1);

    os_by_rust::input::init_keyboard_input();
    os_by_rust::input::push_keyboard_scancode(0x30);
    assert_eq!(os_by_rust::input::pop_keyboard_scancode(), Some(0x30));
    assert_eq!(os_by_rust::input::dropped_scancode_count(), 0);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
