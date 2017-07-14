use nb;

use core::any::{Any};
use core::ops::Deref;
// use core::ptr;

#[cfg(target_arch = "aarch64")]
use rk3399_tools::{I2C0, I2C1, I2C2, I2C3, I2C4, i2c0};

#[cfg(not(target_arch = "aarch64"))]
use rk3399_m0::{I2C0, I2C1, I2C2, I2C3, I2C4, i2c0};

#[derive(Debug)]
pub enum I2CError {
    StartBitTimeout,
    StopBitTimeout,

    /// Communication timeout
    Timeout,

    /// Slave replied to packet with NAK instead of ACK.
    SlaveNak,

    #[doc(hidden)]
    _Extensible,
}

pub type Result<T> = ::core::result::Result<T, nb::Error<I2CError>>;

pub unsafe trait I2CDevice: Deref<Target = i2c0::RegisterBlock> {
    
}

unsafe impl I2CDevice for I2C0 { }
unsafe impl I2CDevice for I2C1 { }
unsafe impl I2CDevice for I2C2 { }
unsafe impl I2CDevice for I2C3 { }
unsafe impl I2CDevice for I2C4 { }

pub trait I2CTrait {
    fn read_from(&self, address: u8, register: Option<u8>, &mut [u8]) -> Result<usize>;
    fn write_to(&self, address: u8, register: Option<u8>, &[u8]) -> Result<usize>;
}

// so, datasheet says max 32 bytes, and I2C code in uboot has the same constant
// however, there are 8 RXDATA registers, so you can get up to 8 * 4 byte = 32 bytes.. oh right.
const I2C_FIFO_SIZE_BYTES: u32 = 32;

// 8th bit in first I2C frame
const RW_BIT_MASTER_READ: u8 = 1;
const RW_BIT_MASTER_WRITE: u8 = 0;

const BITS_PER_BYTE: u32 = 8;

// TODO: fix SVD and have this as an enum
const I2C_MODE_TX: u8 = 0b00;
const I2C_MODE_TRX: u8 = 0b01;
const I2C_MODE_RX: u8  = 0b10;

const TIMEOUT_LOOP: u32 = 10000;

// TODO: investigate how long clock stretching is permitted to happen for.
// If it is a set number of clock cycles, then we can perform an operation blocking
// and then give back some NB error if it times out. Otherwise, each portion
// of the transaction needs to be done separately (or have some dependency on a
// timer).

pub struct I2C<'a, U>(pub &'a U)
where
    U: Any + I2CDevice;

impl<'a, U> I2C<'a, U> where U: Any + I2CDevice {
    fn clear_interrupts(&self) {
        let i2c = self.0;

        // clear pending interrupts (write to clear)
        i2c.rki2c_ipd.write(|w| w.
            btfipd().set_bit().
            brfipd().set_bit().
            mbtfipd().set_bit().
            mbrfipd().set_bit().
            startipd().set_bit().
            stopipd().set_bit().
            nakrcvipd().set_bit().
            slavehdsclipd().set_bit());
    }

    fn send_stop_bit(&self) -> Result<()> {
        self.clear_interrupts();

        let i2c = self.0;

        // enable the module, and generate stop signal
        i2c.rki2c_con.modify(|_, w| w.
            i2c_en().set_bit().
            stop().set_bit());

        // enable interrupt for finished stop
        i2c.rki2c_ien.modify(|_, w| w.
            stopien().set_bit());

        // wait for finish stop interrupt to fire
        // TODO: handle timeout
        let mut attempts = 0;
        while i2c.rki2c_ipd.read().stopipd().bit_is_clear() {
            attempts += 1;

            if attempts > TIMEOUT_LOOP {
                self.disable();
                return Err(nb::Error::Other(I2CError::StopBitTimeout));
            }
        }

        // clear the finish stop interrupt
        i2c.rki2c_ipd.modify(|_, w| w.stopipd().clear_bit());

        Ok(())
    }

    fn send_start_bit(&self) -> Result<()> {
        self.clear_interrupts();

        let i2c = self.0;

        // enable the module, and generate start signal
        i2c.rki2c_con.modify(|_, w| w.
            i2c_en().set_bit().
            start().set_bit());

        // enable interrupt for finished start 
        i2c.rki2c_ien.modify(|_, w| w.
            startien().set_bit());

        // wait for finish start interrupt to fire
        // TODO: handle timeout
        let mut attempts = 0;
        while i2c.rki2c_ipd.read().startipd().bit_is_clear() {
            attempts += 1;

            if attempts > TIMEOUT_LOOP {
                self.disable();
                return Err(nb::Error::Other(I2CError::StartBitTimeout));
            }
        }

        // clear the finish start interrupt
        i2c.rki2c_ipd.modify(|_, w| w.startipd().clear_bit());

        Ok(())
    }

    fn disable(&self) {
        let i2c = self.0;
        i2c.rki2c_con.write(|w| unsafe { w.bits(0) });
    }

    fn terminate(&self) -> Result<()> {
        self.send_stop_bit()?;
        self.disable();
        Ok(())
    }
}

impl<'a, U> Clone for I2C<'a, U>
where
    U: Any + I2CDevice,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U> Copy for I2C<'a, U>
where
    U: Any + I2CDevice,
{
}

impl<'a, U> I2CTrait for I2C<'a, U>
where
    U: Any + I2CDevice,
{
    // type Error = Error;

    /// Read bytes into a slice.
    ///
    /// `address` is a 7-bit I2C address.
    ///
    // For each chunk of data, we tell the I2C controller to enter
    // "TRX" mode (0b01).
    // 
    // It will send the slave's address (stored in the MRXADDR register)
    // onto the bus, with the R/W bit set to 0 (write mode). 
    //
    // Then, the controller instructs the slave device the "register
    // address" it wishes to read from by sending that onto the bus in the
    // following frame. This is stored in the MRXRADDR register.
    //
    // If everything so far has been ACKed, the controller then sends
    // a repeated START sequence automatically, followed by the
    // address + R/W bit, but this time unmodified (i.e. in read mode).
    //
    // The controller now enters "RX" mode (0b10). It releases the SDA
    // line (data), but continues to drive clock. The slave then pulls
    // SDA to send the data to the master. After each byte, the controller
    // will send an ACK, and increment the FIFO register it is storing
    // into.
    //
    // After the expected number of bytes has been received (stored in
    // the MRXCNT register), the master should send a NACK to indicate to
    // the slave that it should stop sending and release SDA.
    //
    // After all data has been transferred, the master should then send
    // a STOP condition to release the bus.
    fn read_from(&self, address: u8, register: Option<u8>, recvdata: &mut [u8]) -> Result<usize> {
        self.send_start_bit()?;

        let i2c = self.0;

        // write the address
        //
        // bottom bit of saddr indicates read or write bit
        // high 7 bits is the address
        // TODO: could support higher than 
        i2c.rki2c_mrxaddr.write(|w| unsafe { w.
            saddr().bits((address << 1 | RW_BIT_MASTER_READ) as u32).
            addlvld().set_bit() // low byte valid
        });

        // write the register address, if provided
        if register != None {
            i2c.rki2c_mrxraddr.write(|w| unsafe { w.
                sraddr().bits(register.unwrap() as u32).
                sraddlvld().set_bit() // low byte valid
            });
        } else {
            // no register addr, set to 0, and mark all u8s invalid
            i2c.rki2c_mrxraddr.reset();
        }

        let mut len = 0;

        // controller can read up to I2C_FIFO_SIZE_BYTES bytes per transaction
        // so, group the buffer into that number of transactions with the I2C controller
        let mut transaction_it = recvdata.chunks_mut(I2C_FIFO_SIZE_BYTES as usize).peekable();

        while transaction_it.peek() != None {
            let transaction_bytes = transaction_it.next().unwrap();

            // setup controller to enter TRX receive mode (see above)
            if transaction_it.peek() == None {
                // last FIFO chunk, so get controller to send a NAK after
                // receive is complete

                i2c.rki2c_con.modify(|_, w| unsafe { w.
                    i2c_en().set_bit().
                    i2c_mode().bits(I2C_MODE_TRX).
                    ack().set_bit() 
                });
            } else {
                // not the last chunk yet
                i2c.rki2c_con.modify(|_, w| unsafe { w.
                    i2c_en().set_bit().
                    i2c_mode().bits(I2C_MODE_TRX)
                });
            }

            // enable "data finished" and "nak handshake" interrupts
            i2c.rki2c_ien.modify(|_, w| w.
                mbrfien().set_bit().
                nakrcvien().set_bit());

            // write out expected receive size
            // controller will attempt reading after this write completes
            i2c.rki2c_mrxcnt.write(|w| unsafe { w.mrxcnt().bits(transaction_bytes.len() as u8) });

            // keep checking for error states or completion
            loop {
                let pending_interrupts = i2c.rki2c_ipd.read();

                // slave replied with NAK; terminate + return error
                if pending_interrupts.nakrcvipd().bit_is_set() {
                    let _ = self.terminate();
                    return Err(nb::Error::Other(I2CError::SlaveNak));
                }

                // receive complete
                if pending_interrupts.mbrfipd().bit_is_set() {
                    break;
                }

                // TODO: handle timeout
            }

            // copy data from FIFO registers into slice
            //
            // note that rxdata buffer is 32-bit, but I2C bus,
            // and thus the API buffers, are 8-bit
            let mut rxbuf_idx = 0;

            for bytes_in_word in transaction_bytes.chunks_mut(4) {
                let rxbytes = i2c.rki2c_rxdata[rxbuf_idx].read().bits();

                for (off, byte) in bytes_in_word.iter_mut().enumerate() {
                    *byte = ((rxbytes >> (off as u32 * BITS_PER_BYTE)) & ((1 << BITS_PER_BYTE) - 1)) as u8;
                    len += 1;
                }

                rxbuf_idx += 1;
            }
        }

        // free up bus
        self.terminate()?;

        // and return number of bytes read
        Ok(len)
    }

    fn write_to(&self, address: u8, register: Option<u8>, data: &[u8]) -> Result<usize> {
        // FIXME: remove this eventually
        assert!(data.len() <= 32 - 2);

        self.send_start_bit()?;

        let i2c = self.0;

        // enable controller and enter TX mode
        i2c.rki2c_con.modify(|_, w| unsafe { w.
            i2c_en().set_bit().
            i2c_mode().bits(I2C_MODE_TX)
        });

        // enable "data finished" and "nak handshake" interrupts
        i2c.rki2c_ien.modify(|_, w| w.
            mbtfien().set_bit().
            nakrcvien().set_bit());

        let address_bytes = match register {
            None => 1,
            Some(_) => 2
        };

        let mut len:u8 = address_bytes;

        let (first_frame, remaining_frames) = match data.len() > address_bytes as usize {
            true => {
                let (a, b) = data.split_at(address_bytes as usize);
                (a, Some(b))
            },
            _    => (data, None)
        };

        // first tx register is special case; it contains the slave address
        // and possibly the register address
        let mut tx_0 = (address << 1 | RW_BIT_MASTER_WRITE) as u32;
        if register != None {
            tx_0 |= (register.unwrap() as u32) << 8;
        }

        // remaining 2 bytes for first tx register
        for (idx, byte) in first_frame.iter().enumerate() {
            tx_0 |= (*byte as u32) << (8 * (idx + 2));
            len += 1;
        }

        i2c.rki2c_txdata[0].write(|w| unsafe { w.bits(tx_0) });

        // do any remaining tx registers
        if remaining_frames != None {
            let txreg_idx = 1;

            for bytes_in_word in remaining_frames.unwrap().chunks(4) {
                let mut bitstuffed:u32 = 0;
                for (off, byte) in bytes_in_word.iter().enumerate() {
                    bitstuffed = bitstuffed | ((*byte as u32) << (off as u32 * BITS_PER_BYTE));
                    len += 1;
                }

                i2c.rki2c_txdata[txreg_idx].write(|w| unsafe { w.bits(bitstuffed) });
            }
        }

        // write out tx length; this initiates transfer
        i2c.rki2c_mtxcnt.write(|w| unsafe { w.mtxcnt().bits(len) });    
    
        // keep checking for error states or completion
        // TODO: move into separate function
        let mut attempts = 0;
        loop {
            let pending_interrupts = i2c.rki2c_ipd.read();

            // slave replied with NAK; terminate + return error
            if pending_interrupts.nakrcvipd().bit_is_set() {
                let _ = self.terminate();
                return Err(nb::Error::Other(I2CError::SlaveNak));
            }

            // transmission complete
            if pending_interrupts.mbtfipd().bit_is_set() {
                break;
            }

            // TODO: handle timeout
            attempts += 1;

            if attempts > TIMEOUT_LOOP {
                let _ = self.terminate();
                return Err(nb::Error::Other(I2CError::Timeout));  
            }
        }

        // free up bus
        self.terminate()?;

        // and return number of bytes read
        Ok(len as usize)
    }
}
