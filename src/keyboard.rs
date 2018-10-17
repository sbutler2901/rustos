use spin::Mutex;

lazy_static! {
    static ref SHIFT_ACTIVE: Mutex<bool> = {
        let shift_active = false;
        Mutex::new(shift_active)
    };
}

fn transform(key: Option<char>) -> Option<char>{
    if let Some(key_mapped) = key {
        // Convert ascii letter to uppercase if shift key is active, does nothing if not letter
        if *SHIFT_ACTIVE.lock() {
            return Some(key_mapped.to_ascii_uppercase());
        }
    }
    key
}

/// PS/2 Scancode Set 1 mapping
pub fn scancode_map(scancode: u8) -> Option<char> {
    let key = match scancode {
        0x01 => None,       // escape
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        0x0E => None,       // backspace
        0x0F => None,       // tab
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => None,       // enter
        0x1D => None,       // left ctrl
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),
        0x2A => {           // left shift pressed
            *SHIFT_ACTIVE.lock() = true;
            None
        },
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x36 => {           // right shift pressed
            *SHIFT_ACTIVE.lock() = true;
            None
        },
        0x37 => None,       // (keypad) * pressed
        0x38 => None,       // left alt
        0x39 => Some(' '),
        0x3A => None,       // Capslock pressed
        0xAA => {           // left shift released
            *SHIFT_ACTIVE.lock() = false;
            None
        }
        0xB6 => {           // right shift released
            *SHIFT_ACTIVE.lock() = false;
            None
        }
        _ => None,
    };
    transform(key)
}
