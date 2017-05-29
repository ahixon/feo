use core::ptr::{read_volatile, write_volatile};
use core::fmt;
use core::ptr::Unique;

// const RBR:u32 	= 0x000;
const THR:u32 	= 0x000;
const LSR:u32 	= 0x014;
const FCR:u32   = 0x008;

const THR_EMPTY_BIT:u32 = (1 << 5);

const FCR_FIFO_ENABLE:u32 = (1 << 0);
const FCR_RECV_FIFO_RESET:u32 = (1 << 1);

pub struct Uart16650 {
	pub base: Unique<u8>
	// pub base: *mut u8
}

impl Uart16650 {
	pub fn new(base: Unique<u8>) -> Uart16650 {
		// setup FIFO
		unsafe {
			let fifo_control_register_ptr = (base.offset(FCR as isize)) as *mut u32;
			write_volatile(fifo_control_register_ptr, FCR_FIFO_ENABLE | FCR_RECV_FIFO_RESET);
		}
		
	    return Uart16650 {
	    	base: base
	    }
	}
}

impl fmt::Write for Uart16650 {
	/// The `fmt::Write` trait requires that this function
	/// not return until the entire bytestring been written.
	///
	/// If the UART's FIFO fills up while writing the string,
	/// then we will busy-wait until it clears again.
    fn write_str(&mut self, s: &str) -> fmt::Result {
    	unsafe {
	    	let data_register_ptr = (self.base.offset(THR as isize)) as *mut u32;
	    	let status_register_ptr = (self.base.offset(LSR as isize)) as *mut u32;

	        for byte in s.bytes() {
	        	// wait until ready to transmit (bit goes high when empty)
	          	while read_volatile(status_register_ptr) & THR_EMPTY_BIT == 0 { }

				// move to transmit holding register
				write_volatile(data_register_ptr, byte as u32);
	        }
	    }

        Ok(())
    }
}