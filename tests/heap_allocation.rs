#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os_by_rust::{allocator::HEAP_SIZE, memory::BootInfoFrameAllocator};
use x86_64::PhysAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use os_by_rust::allocator;
    use os_by_rust::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    os_by_rust::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { os_by_rust::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed!");

    test_main();
    loop {}
    //unimplemented!();
}

#[test_case]
// 测试是否未发生分配错误
fn simple_allocation() {
    let heap_val_1 = Box::new(42);
    let heap_val_2 = Box::new(1337);
    assert_eq!(*heap_val_1, 42);
    assert_eq!(*heap_val_2, 1337);
}

// 测试大型分配
#[test_case]
fn large_vec() {
    let n = 1001;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.len(), n);
    //assert_eq!(vec.iter().sum::<u64>(), n * (n - 1) / 2);
    assert_eq!(
        vec.iter().map(|&x| x as u64).sum::<u64>(),
        (n * (n - 1) / 2).try_into().unwrap()
    );
}

// 测试多重分配
// 此测试可确保分配器将释放的内存重新用于后续分配，否则它将耗尽内存
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

// bump allocator对于该测试会出现错误
// 手写的ListNodeAllocator能够将释放的内存复用到后续的分配中, 可通过该测试
#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info);
}
