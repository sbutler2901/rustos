use uart_16550::SerialPort;
use spin::Mutex;

// lazy_static and spinlock used to reate a static reference to serial I/O port
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // standard port address for first serial interface: 0x3F8
        let mut serial_port = SerialPort::new(0x3F8);
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// fmt::Write trait already implemented for type SerialPort
pub fn print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => (serial_print!("\n"));
    ($fmt:expr) => (serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (serial_print!(concat!($fmt, "\n"), $($arg)*));
}

