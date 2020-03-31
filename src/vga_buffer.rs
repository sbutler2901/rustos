use core::fmt;
use volatile::Volatile;
use spin::Mutex;

#[allow(dead_code)]     // prevents compiler warnings that some enumerations are never used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]        // enables copy semantics for the type: makes printable & comparable
#[repr(u8)]     // makes each enum variant be stored as a u8
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

// used to represent a full VGA color code (foreground & background)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);       // creates a type which is essentially an alias for a single byte

impl ColorCode {
    // creates a single byte detailing the fore and background colors (based on VGA specifications)
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// ensures struct's field laid out exactly like a C struct since VGA depends on the order of the two bytes
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,        // VGA byte representing ascii char
    color_code: ColorCode,      // VGA byte representing char's color
}

// VGA typical buffer sizes
const BUFFER_HEIGHT: usize = 25;        // number of lines
const BUFFER_WIDTH: usize = 80;         // number of chars in line

struct Buffer {
    // Volatile crate keeps rust compiler from optimizing and removing writes
    // since writes are never read and are going to the VGA buffer memory (a side-effect)
    // and not just writing to RAM
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// To actually write to screen: always writes to last line & shift lines up when a line is full (or on \n)
pub struct Writer {
    column_position: usize,             // keeps track of current position in last row
    color_code: ColorCode,              // current fore & background colors
    buffer: &'static mut Buffer,        // reference to VGA buffer: 'static lifetime specifies reference is valid for whole program run time (VGA buffer)
}

impl Writer {
    // writes a single byte to the screen at current location
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
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    // accepts a string to be written only writing valid ascii chars
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }

    fn new_line(&mut self) {
        // range notation is exclusive of upper end.
        // top line of screen is 0 and is shifted off screen
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(char);
            }
        }
        // clears last line of output for new input, otherwise if string being written
        // is not long enough all previous characters will not be overwritten
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // clears row by overwriting characters with spaces
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// Provides support for Rust's formatting macros allowing easy printing
// of different types like integers or floats.
// Results in: Write! / Writeln! macro support
impl fmt::Write for Writer {
    // The only required method of the fmt::Write trait
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Provides a static Writer object which utilizes non-const functions
// Requires locking to provide interior mutability: since it utilizes &mut self for writing
// it requires mutability, but its mutibility is not provided to users, therefore it is interior
// mutability. The Mutex allows safe usage internally.
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        // provides a direct mutable reference to the VGA memory-mapped I/O address
        // allowing reading and writing. We deem this safe as this address always corresponds to
        // VGA, and therefore it is acceptable and required to wrap in an unsafe block
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[cfg(test)]
mod test {
    use super::*;       // import all items of parent module: vga_buffer

    // Specifies what char represents an empty cell in VGA buffer during testing
    fn empty_char() -> ScreenChar {
        ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::Green, Color::Brown),
        }
    }

    fn construct_buffer() -> Buffer {
        // bypasses array construction requiring that contained type is Copy
        // ScreenChar satisfies this, but the Volatile wrapper does not
        use array_init::array_init;

        Buffer {
            // Provides array initialization without non-Copy types.
            // parameter of array_init is a closure. The single parameter to the closure is unused and therefore unimportant
            // otherwise it could be used to perform calculations on value before creating the array.
            // array_init utilizes type's size to create the required number of indices. In this case
            // the number of columns and rows are defined in the Buffer struct
            // "The width & height are deduced by type inference"
            chars: array_init(|_| array_init(|_| Volatile::new(empty_char()))),
        }
    }

    fn construct_writer() -> Writer {
        use std::boxed::Box;

        let buffer = construct_buffer();
        Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Blue, Color::Magenta),
            // transforms the created buffer into a &'static mut to satisfy buffer property's type
            buffer: Box::leak(Box::new(buffer)),
        }
    }

    #[test]             // tells test framework this is a test function
    fn write_byte() {
        let mut writer = construct_writer();
        writer.write_byte(b'X');
        writer.write_byte(b'Y');

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == BUFFER_HEIGHT - 1 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'X');
                    assert_eq!(screen_char.color_code, writer.color_code);

                } else if i == BUFFER_HEIGHT - 1 && j == 1 {
                    assert_eq!(screen_char.ascii_character, b'Y');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }

    #[test]
    fn write_formatted() {
        use core::fmt::Write;

        let mut writer = construct_writer();
        writeln!(&mut writer, "a").unwrap();
        writeln!(&mut writer, "b{}", "c").unwrap();

        for (i, row) in writer.buffer.chars.iter().enumerate() {
            for (j, screen_char) in row.iter().enumerate() {
                let screen_char = screen_char.read();
                if i == BUFFER_HEIGHT - 3 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'a');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 0 {
                    assert_eq!(screen_char.ascii_character, b'b');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i == BUFFER_HEIGHT - 2 && j == 1 {
                    assert_eq!(screen_char.ascii_character, b'c');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else if i >= BUFFER_HEIGHT - 2 {              // ensures empty lines are shifted in on a new line and have correct color code
                    assert_eq!(screen_char.ascii_character, b' ');
                    assert_eq!(screen_char.color_code, writer.color_code);
                } else {
                    assert_eq!(screen_char, empty_char());
                }
            }
        }
    }
}
