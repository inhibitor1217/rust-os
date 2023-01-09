use volatile::Volatile;

/// Colors available in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // https://en.wikipedia.org/wiki/VGA_text_mode
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
struct ColorCode(u8); // https://en.wikipedia.org/wiki/VGA_text_mode

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct TextCharacter {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct TextBuffer {
    chars: [[Volatile<TextCharacter>; TextBuffer::WIDTH]; TextBuffer::HEIGHT],
}

impl TextBuffer {
    const WIDTH: usize = 80;
    const HEIGHT: usize = 25;

    fn set(&mut self, row: usize, col: usize, char: TextCharacter) {
        self.chars[row][col].write(char);
    }
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    text_buffer: &'static mut TextBuffer,
}

impl Writer {
    /// Write a single ASCII byte to VGA text buffer.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= TextBuffer::WIDTH {
                    // auto shift line
                    self.new_line();
                }

                // always write to the last line
                let row = TextBuffer::HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.text_buffer.set(
                    row,
                    col,
                    TextCharacter {
                        ascii_character: byte,
                        color_code,
                    },
                );
                self.column_position += 1;
            }
        }
    }

    /// Write a string slice to VGA text buffer.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII bytes
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // non-printable bytes
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        todo!()
    }
}

/// temporary
pub fn print_foo() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        text_buffer: unsafe { &mut *(0xb8000 as *mut TextBuffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}
