use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame}; // 可以让变量在第一次取值时进行初始化而不是在编译时，避免数据竞争

// 理论上 static mut 类型的变量很容易形成数据竞争，所以需要用 unsafe 代码块 修饰调用语句
//static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();

    // unsafe {
    //     //let mut idt = InterruptDescriptorTable::new();
    //     IDT.breakpoint.set_handler_fn(breakpoint_handler);
    //     IDT.load();
    // }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // 触发断点异常
    x86_64::instructions::interrupts::int3();
}
