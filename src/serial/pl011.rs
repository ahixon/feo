use core::ptr::{read_volatile, write_volatile};

const UARTDR:u32 	= 0x000;
// const UARTRSR:u32	= 0x004;
// const UARTECR:u32 	= 0x004;
const UARTFR:u32 	= 0x018;

pub struct PL011 {
	pub base: u32
}

bitflags! {
	flags PL011Flags: u32 {
		const RING_INDICATOR = 		0b10000000,	// RI
		const TRANSMIT_FIFO_EMPTY = 	0b01000000, // TXFE
		const RECEIVE_FIFO_FULL =	0b00100000, // RXFF
		const TRANSMIT_FIFO_FULL =   0b00010000, // TXFF
		const RECEIVE_FIFO_EMPTY =	0b00001000, // RXFE
		const BUSY =				0b00000100, // BUSY
		const DATA_CARRIER_DETECT =  0b00000010, // DCD
		const DATA_SET_READY =       0b00000001, // DSR
		const CLEAR_TO_SEND =        0b00000000  // CTS
	}
}


// we can write characters individually to the buffer
// and transmit as we go

// or, we can do a DMA transfer
// it can either initiate it, or we can
// DMA interrupt signals are wired to UART controller directly

impl PL011 {
	fn get_flags(&self) -> PL011Flags {
		let status_register_ptr = (self.base + UARTFR) as *const u32;

		unsafe {
			PL011Flags::from_bits_truncate (read_volatile::<u32> (status_register_ptr))
		}
	}

	// a direct hardware write
	// if the fifo buffer fills up, then we stop processing
	// maybe have controllable busy-wait?
	pub fn write(&self, buf: &[u8]) -> Result<usize, ()> {
		let mut total_written = 0;

		for (written, byte) in buf.iter().enumerate() {
			let data_register_ptr = (self.base + UARTDR) as *mut u32;

			unsafe {
				// move to data register
				write_volatile::<u32>(data_register_ptr, *byte as u32);
			}

			// ensure FIFO isn't full
			//
			// we could also receive an interrupt
			// when the FIFO fills up and wait until it
			// empties again...
			let flags = self.get_flags ();
			if flags.contains (TRANSMIT_FIFO_FULL) {
				return Ok(written + 1);
			}

			total_written = written;
		}

		Ok(total_written)
	}
}