use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).unwrap();
}

/// Prints to the host through the serial interface.
#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! serial_print {
  ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

/// Prints to the host through the serial interface, appending a newline.
#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! serial_println {
  () => ($crate::serial_print!("\n"));
  ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
