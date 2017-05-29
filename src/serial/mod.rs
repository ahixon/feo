use core::ptr::Unique;
use spin::Mutex;
use core::fmt;

pub mod pl011;
pub use self::pl011::PL011;

pub mod uart16650;
pub use self::uart16650::Uart16650;

// can't do lazy_static because no std
// so we create the struct manually
// thankfully setup's been done for us by uboot...
pub static STDOUT: Mutex<Uart16650> = Mutex::new(Uart16650 {
	base: unsafe { 
		Unique::new (0xFF1A0000 as *mut u8)  // UART2
	}
});

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::serial::print(format_args!($($arg)*));
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    STDOUT.lock().write_fmt(args).unwrap();
}