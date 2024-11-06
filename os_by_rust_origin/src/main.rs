#![no_std] //为构建一颗操作系统内核，我们需要避免使用依赖os的标准库
#![no_main] // 不使用预定义入口点
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer; // 串口模块

use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// 定义一个空的 panic 处理函数，以防止程序崩溃
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 操作系统的入口点
    //在自定义println!宏后，打印信息到vga缓冲区
    println!("Hello World, again{}", "!");
    panic!("some os panic message");

    #[cfg(test)]
    test_main();

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again!").unwrap();
    // write!(
    //     vga_buffer::WRITER.lock(),
    //     ", some nums: {}, {}, {}",
    //     22,
    //     1.332,
    //     7.0
    // )
    // .unwrap();

    //vga_buffer::print_something();

    // let vga_buffer = 0xb8000 as *mut u8; //vga 硬件入口地址

    // for(i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte; // 写入字符到vga 缓冲区
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // 设置颜色
    //     }
    // }

    loop {}
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
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    //assert_eq!(0, 1);
    serial_println!("[ok]");
}

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
