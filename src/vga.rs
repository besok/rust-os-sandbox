use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::default());
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
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

impl Default for ColorCode {
    fn default() -> Self {
        ColorCode::new(Color::White, Color::Black)
    }
}

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    ch: u8,
    color: ColorCode,
}

const H: usize = 25;
const W: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<Char>; W]; H],
}

pub struct Writer {
    pos: usize,
    color: ColorCode,
    buf: &'static mut Buffer,
}

impl Default for Writer {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            color: Default::default(),
            buf: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.pos >= W {
                    self.new_line();
                }

                let row = H - 1;
                let col = self.pos;

                let color = self.color;
                self.buf.chars[row][col].write(Char { ch: byte, color });
                self.pos += 1;
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..H {
            for col in 0..W {
                let character = self.buf.chars[row][col].read();
                self.buf.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(H - 1);
        self.pos = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            ch: b' ',
            color: self.color,
        };
        for col in 0..W {
            self.buf.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buf.chars[H - 2][i].read();
        assert_eq!(char::from(screen_char.ch), c);
    }
}
