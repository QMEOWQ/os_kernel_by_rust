use volatile::Volatile;

#[allow(dead_code)] // 禁用每个未使用的变量发出警告
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // 为了确保 ColorCode 和 u8 有完全相同的内存布局

pub enum Color {
    // Black = 0,
    // White = 1,
    // Red = 2,
    // Green = 3,
    // Blue = 4,
    // Yellow = 5,
    // Cyan = 6,
    // Magenta = 7,
    // Brown = 8,
    // LightGray = 9,
    // DarkGray = 10,
    // LightRed = 11,
    // LightGreen = 12,
    // LightBlue = 13,
    // LightCyan = 14,
    // Pink = 15,
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]

struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // 按 C 语言约定的顺序布局它的成员变量，让我们能正确地映射内存片段

struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// struct Buffer {
//     chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
// }

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // self.buffer.chars[row][col] = ScreenChar {
                //     ascii_character: byte,
                //     color_code,
                // };
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                //不包含在上述范围之内的字节
                _ => self.write_byte(0xfe), // 打印符号■表示非法字符
            }
        }
    }

    fn new_line(&mut self) {
        //todo!()
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character)
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    //清空一整行的字符位置
    fn clear_row(&mut self, row: usize) {
        //todo!()
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

//测试Writer中的方法
pub fn print_something() {
    use core::fmt::Write;

    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello! ");
    write!(writer, "The numbers are {} and {}", 22, 1.0 / 3.0).unwrap();
}

// pub fn print_something() {
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Yellow, Color::Black),
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };

//     writer.write_byte(b'H');
//     writer.write_string("ello ");
//     writer.write_string("World!");
//     writer.write_string("Wörld!");
// }
