//! A VGA text-mode driver

use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;
use core::fmt;

lazy_static!(
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col_position: 0,
        color: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
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

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
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

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> Self {
        Self((bg as u8) << 4 | (fg as u8))
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer ([[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]);

pub struct Writer {
    col_position: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Write a single byte to the VGA text-mode output, going
    /// to the next line if necessary or `\n` was input
    fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' { self.new_line(); return }

        if self.col_position >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT-1;
        let col = self.col_position;
        let color = self.color;

        self.buffer.0[row][col].write(ScreenChar {
            ascii_char: byte,
            color_code: color,
        });
        self.col_position += 1;
    }

    /// Write a string to the VGA text-mode output
    pub fn write_string(&mut self, string: &str) {
        for ch in string.bytes() {
            match ch {
                // Printable ASCII character
                0x20..=0x7E | b'\n' => self.write_byte(ch),
                // Non-printable ASCII character
                _ => self.write_byte(0xFE),
            }
        }
    }

    /// Insert a new line, moving all other lines up
    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.0[row][col].read();
                self.buffer.0[row-1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT-1);
        self.col_position = 0;
    }

    /// Clear the given row
    pub fn clear_row(&mut self, row: usize) {
        for i in self.buffer.0[row].iter_mut() {
            i.write(ScreenChar {
                ascii_char: b' ',
                color_code: self.color,
            });
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test_case]
    fn println_simple() {
        println!("println_simple output");
    }

    #[test_case]
    fn println_many() {
        for _ in 0..200 {
            println!("println_many output");
        }
    }

    #[test_case]
    fn println_output() {
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = super::WRITER.lock().buffer.0[super::BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_char), c);
        }
    }
}