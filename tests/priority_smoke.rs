#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU8, Ordering};
use os_by_rust::task::executor::Executor;
use os_by_rust::task::{Task, TaskPriority};
use x86_64::VirtAddr;

static EXECUTION_ORDER: AtomicU8 = AtomicU8::new(0);

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use os_by_rust::allocator;
    use os_by_rust::memory::{self, BootInfoFrameAllocator};

    os_by_rust::init();
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new();
    // 故意先插入普通任务，再插入高优任务，验证高优任务会先执行。
    executor
        .try_spawn(Task::new_with_priority(
            normal_priority_task(),
            TaskPriority::Normal,
        ))
        .expect("failed to spawn normal task");
    executor
        .try_spawn(Task::new_with_priority(
            high_priority_task(),
            TaskPriority::High,
        ))
        .expect("failed to spawn high priority task");
    executor.run();
}

async fn high_priority_task() {
    EXECUTION_ORDER.store(1, Ordering::Relaxed);
}

async fn normal_priority_task() {
    assert_eq!(
        EXECUTION_ORDER.load(Ordering::Relaxed),
        1,
        "high priority task should run first",
    );
    os_by_rust::exit_qemu(os_by_rust::QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
