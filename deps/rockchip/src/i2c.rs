use nb;

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

use rk3399_tools::{I2C0, i2c0};

pub enum I2CError {
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

unsafe impl I2CDevice for I2C0 {

}

pub trait I2CTrait {
    fn read_from(&self, address: u8, register: u8, &mut [u8]) -> Result<usize>;
}

const I2C_FIFO_SIZE_BYTES: u32 = 32;

// 8th bit in first I2C frame
const RW_BIT_MASTER_READ: u8 = 1;
const RW_BIT_MASTER_WRITE: u8 = 0;

const BITS_PER_BYTE: u32 = 8;

// TODO: fix SVD and have this as an enum
const I2C_MODE_TRX: u8 = 0b01;

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
        while i2c.rki2c_ipd.read().stopipd().bit_is_clear() {

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
        while i2c.rki2c_ipd.read().startipd().bit_is_clear() {}

        // clear the finish start interrupt
        i2c.rki2c_ipd.modify(|_, w| w.startipd().clear_bit());

        Ok(())
    }

    fn disable(&self) {
        let i2c = self.0;
        i2c.rki2c_con.write(|w| unsafe { w.bits(0) });
    }

    fn terminate(&self) {
        self.send_stop_bit();
        self.disable();
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
    fn read_from(&self, address: u8, register: u8, recvdata: &mut [u8]) -> Result<usize> {
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
        if register > 0 {
            i2c.rki2c_mrxraddr.write(|w| unsafe { w.
                sraddr().bits(register as u32).
                sraddlvld().set_bit() // low byte valid
            });
        } else {
            // no register addr, set to 0, and mark all u8s invalid
            i2c.rki2c_mrxraddr.reset();
        }

        // controller can read up to I2C_FIFO_SIZE_BYTES bytes per transaction
        let len = recvdata.len();

        let mut it = recvdata.chunks_mut((I2C_FIFO_SIZE_BYTES / 4) as usize).peekable();
        let mut rxbuf_idx = 0;

        while it.peek() != None {
            let chunk = it.next().unwrap();

            // setup controller to enter TRX receive mode (see above)
            if it.peek() == None {
                // last chunk, so get controller to send a NAK after
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
            i2c.rki2c_mrxcnt.write(|w| unsafe { w.mrxcnt().bits(chunk.len() as u8) });

            // keep checking for error states or completion
            loop {
                let pending_interrupts = i2c.rki2c_ipd.read();

                // slave replied with NAK; terminate + return error
                if pending_interrupts.nakrcvipd().bit_is_set() {
                    self.terminate();
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
            for (off, byte) in chunk.iter_mut().enumerate() {
                let rxbytes = i2c.rki2c_rxdata[rxbuf_idx].read().bits();
                *byte = ((rxbytes >> (off as u32 * BITS_PER_BYTE)) & ((1 << BITS_PER_BYTE) - 1)) as u8;
            }

            rxbuf_idx += 1;
        }

        // free up bus
        self.terminate();

        // and return number of bytes read
        Ok(len)
    }
}
