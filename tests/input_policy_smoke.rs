#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use os_by_rust::allocator;
    use os_by_rust::memory::{self, BootInfoFrameAllocator};

    os_by_rust::init();
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    os_by_rust::input::init_keyboard_input();
    test_main();
    os_by_rust::exit_qemu(os_by_rust::QemuExitCode::Success);
    os_by_rust::hlt_loop();
}

#[cfg(feature = "input-drop-new")]
#[test_case]
fn input_drop_new_policy_keeps_oldest_queue_entries() {
    os_by_rust::input::reset_counters_for_test();
    drain_scancode_queue();

    for value in 0u8..120u8 {
        os_by_rust::input::push_keyboard_scancode(value);
    }

    assert_eq!(os_by_rust::input::dropped_scancode_count(), 20);
    assert_eq!(os_by_rust::input::pop_keyboard_scancode(), Some(0));
}

#[cfg(feature = "input-drop-old")]
#[test_case]
fn input_drop_old_policy_keeps_latest_queue_entries() {
    os_by_rust::input::reset_counters_for_test();
    drain_scancode_queue();

    for value in 0u8..120u8 {
        os_by_rust::input::push_keyboard_scancode(value);
    }

    assert_eq!(os_by_rust::input::dropped_scancode_count(), 0);
    assert_eq!(os_by_rust::input::pop_keyboard_scancode(), Some(20));
}

fn drain_scancode_queue() {
    while os_by_rust::input::pop_keyboard_scancode().is_some() {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
