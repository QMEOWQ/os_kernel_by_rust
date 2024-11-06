#![no_std] //为构建一颗操作系统内核，我们需要避免使用依赖os的标准库
#![no_main] // 不使用预定义入口点
#![feature(custom_test_frameworks)]
#![test_runner(os_by_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os_by_rust::{memory::translate_addr, println};
use x86_64::structures::paging::{page, page_table::PageTableEntry, PageTable};

// 确保入口点函数总是具有引导程序所期望的正确签名
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // 操作系统的入口点
    use os_by_rust::memory;
    use x86_64::{structures::paging::Page, structures::paging::Translate, VirtAddr};

    //在自定义println!宏后，打印信息到vga缓冲区
    println!("Hello World{}", "!");

    os_by_rust::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut _mapper = unsafe { memory::init(phys_mem_offset) };
    let mut _frmae_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    //let mut frame_allocator = memory::EmptyFrameAllocator;

    // 映射未使用的页
    // 创建该映射是因为负责地址为0的页面的1级表已经存在, 否则会失败
    // let page = Page::containing_address(VirtAddr::new(0));
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // 通过新的映射将字符串 `New!`  写到屏幕上
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     // 此处调用的translate_addr为x86 translate包中的函数，并非memory.rs中自定义函数
    //     let phys = mapper.translate_addr(virt);
    //     //let phys = unsafe { translate_addr(virt, phys_mem_offset) };
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     // 只打印l4非空条目
    //     if !entry.is_unused() {
    //         println!("L4 entry {}: {:#x?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         for (i, entry) in l3_table.iter().enumerate() {
    //             // 打印l3非空条目, l2 l1 同理
    //             if !entry.is_unused() {
    //                 println!("L3 entry {}: {:#x?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // let (level_4_page_table, _) = Cr3::read();
    // println!(
    //     "Level 4 page table at: {:?}",
    //     level_4_page_table.start_address()
    // );

    // 尝试访问一个无效的地址
    //let ptr = 0xdeadbeef as *mut u8;
    // unsafe {
    //     *ptr = 42;
    // }

    // 正常情况下只能看到read worded, 因为我们之前为其设置了只读
    // let ptr = 0x2041bc as *mut u8;

    // // read from a code page
    // unsafe {
    //     let x = *ptr;
    // }
    // println!("read worked");

    // // write to a code page
    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked");

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os_by_rust::hlt_loop();
}

// #[no_mangle]
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
//     // 操作系统的入口点
//     //在自定义println!宏后，打印信息到vga缓冲区
//     println!("Hello World, again{}", "!");

//     os_by_rust::init();

//     use x86_64::registers::control::Cr3;

//     let (level_4_page_table, _) = Cr3::read();
//     println!(
//         "Level 4 page table at: {:?}",
//         level_4_page_table.start_address()
//     );

//     // 尝试访问一个无效的地址
//     //let ptr = 0xdeadbeef as *mut u8;
//     // unsafe {
//     //     *ptr = 42;
//     // }

//     // 正常情况下只能看到read worded, 因为我们之前为其设置了只读
//     // let ptr = 0x2041bc as *mut u8;

//     // // read from a code page
//     // unsafe {
//     //     let x = *ptr;
//     // }
//     // println!("read worked");

//     // // write to a code page
//     // unsafe {
//     //     *ptr = 42;
//     // }
//     // println!("write worked");

//     #[cfg(test)]
//     test_main();

//     println!("It did not crash!");
//     os_by_rust::hlt_loop();
// }

// 定义一个空的 panic 处理函数，以防止程序崩溃
// 使用条件编译（conditional compilation）在测试模式下使用（与非测试模式下）不同的panic处理方式
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os_by_rust::hlt_loop();
    //loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_by_rust::test_panic_handler(info)
}

//实现自定义测试框架  cargo test
// #![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]

//测试exit_qemu
// #[cfg(test)]
// fn test_runner(tests: &[&dyn Fn()]) {
//     println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//     }

//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
// fn trivial_assertion() {
//     print!("trivial assertion...");
//     assert_eq!(1, 1);
//     println!("[ok]");
// }

//测试串口打印
// #[cfg(test)]
// // 实现自动打印测试结果
// fn test_runner(tests: &[&dyn Testable]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test.run();
//     }

//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
// fn trivial_assertion() {
//     assert_eq!(1, 1);
//     //assert_eq!(0, 1);

//     // 会根据test-timeout的值来判断超时，默认300s
//     loop {}
// }

// #[test_case]
// fn test_println_simple() {
//     println!("test_println_simple output");
// }

// #[test_case]
// fn test_println_many() {
//     for _ in 0..200 {
//         println!("test_println_many output");
//     }
// }

// fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//     }

//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
// fn trivial_assertion() {
//     serial_print!("trivial assertion... ");
//     assert_eq!(1, 1);
//     //assert_eq!(0, 1);
//     serial_println!("[ok]");

//     // 会根据test-timeout的值来判断超时，默认300s
//     loop {}
// }

// fn main() {}

// fn main() {
//     println!("Hello, world!");
// }

// rustc --version --verbose 查看当前宿主机器信息
// rustup target add thumbv7em-none-eabihf 描述了一个ARM嵌入式系统,由none知环境底层没有操作系统
// cargo build --target thumbv7em-none-eabihf 编译当前项目为ARM嵌入式系统的可执行文件

// 以下两条命令为选择本地操作系统为目标进行编译
// # Linux
// cargo rustc -- -C link-arg=-nostartfiles
// # Windows
// cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"

// cargo build --target x86_64-os_by_rust.json

// rustup component add rust-src

// cargo install bootimage
// rustup component add llvm-tools-preview
// cargo bootimage 编译自定义内核

// 第一次编译自定义内核后，之后使用 cargo build/run编译构建

// 1. 在虚拟机中启动内核
// qemu-system-x86_64 -drive format=raw,file=target/x86_64-os_by_rust/debug/bootimage-os_by_rust.bin
// 2. 在真机上运行内核
// dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
// 其中 sdX为U盘设备名，在选择设备名的时候一定要极其小心，因为目标设备上已有的数据将全部被擦除。

// in Cargo.toml
// [profile.dev] #对应cargo build命令
// panic = "abort"

// [profile.release] #对应cargo build --release命令
// panic = "abort"

/*
即使我们传递了表示成功（Success）的退出代码,
cargo test 依然会将所有的测试都视为失败,
这里的问题在于，cargo test 会将所有非 0 的错误码都视为测试失败。
在Cargo.toml添加以下配置test-success-exit-code = 33即可,
有了这个配置，bootimage 就会将我们的成功退出码映射到退出码0；
这样一来， cargo test 就能正确地识别出测试成功的情况，
而不会将其视为测试失败。
 */
