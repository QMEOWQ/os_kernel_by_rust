#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::future::Future;
use core::panic::PanicInfo;
use core::pin::Pin;
use core::task::{Context, Poll};
use os_by_rust::task::executor::Executor;
use os_by_rust::task::Task;
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

    let mut executor = Executor::new();
    executor
        .try_spawn(Task::new(executor_smoke_task()))
        .expect("failed to spawn executor smoke task");
    executor.run();
}

struct YieldMultiple {
    remaining_yields: usize,
}

impl YieldMultiple {
    fn new(yield_count: usize) -> Self {
        Self {
            remaining_yields: yield_count,
        }
    }
}

impl Future for YieldMultiple {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining_yields == 0 {
            Poll::Ready(())
        } else {
            self.remaining_yields -= 1;
            // 重复触发wake，覆盖“重复唤醒入队”路径。
            context.waker().wake_by_ref();
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

async fn executor_smoke_task() {
    YieldMultiple::new(16).await;
    os_by_rust::exit_qemu(os_by_rust::QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
