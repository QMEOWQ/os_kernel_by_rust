#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;
use bootloader::{entry_point, BootInfo};
use core::future::Future;
use core::panic::PanicInfo;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
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

    test_main();
    os_by_rust::exit_qemu(os_by_rust::QemuExitCode::Success);
    os_by_rust::hlt_loop();
}

#[test_case]
fn sleep_ticks_wakes_after_target_tick() {
    let mut sleep_future = Box::pin(os_by_rust::task::sleep_ticks(2));
    let noop_waker = NoopWake::waker();
    let mut context = Context::from_waker(&noop_waker);

    assert!(matches!(
        Pin::as_mut(&mut sleep_future).poll(&mut context),
        Poll::Pending
    ));

    os_by_rust::task::timer::tick();
    assert!(matches!(
        Pin::as_mut(&mut sleep_future).poll(&mut context),
        Poll::Pending
    ));

    os_by_rust::task::timer::tick();
    assert!(matches!(
        Pin::as_mut(&mut sleep_future).poll(&mut context),
        Poll::Ready(())
    ));
}

struct NoopWake;

impl NoopWake {
    fn waker() -> Waker {
        Waker::from(Arc::new(Self))
    }
}

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
