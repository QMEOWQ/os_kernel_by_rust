use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// 像VGA文本缓冲区一样，使用 lazy_static 和一个自旋锁来创建一个 static writer实例
// 通过使用 lazy_static ，我们可以保证 init 方法只会在该示例第一次被使用使被调用
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // 0x3F8: 第一个串行接口的标准端口号。
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    //在串行输出函数里也加入同样的逻辑来避免死锁
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg: tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
