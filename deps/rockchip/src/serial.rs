use hal;
use nb;

use core::any::{Any, TypeId};
use core::ops::Deref;
use core::ptr;

#[cfg(target_arch = "aarch64")]
use rk3399_tools::{UART0, uart0};

#[cfg(not(target_arch = "aarch64"))]
use rk3399_m0::{UART0, uart0};

pub type Result<T> = ::core::result::Result<T, nb::Error<Error>>;

pub unsafe trait Usart: Deref<Target = uart0::RegisterBlock> {

}

unsafe impl Usart for UART0 {

}

/// An error
#[derive(Debug)]
pub enum Error {
    /// RX buffer overrun
    Overrun,
    #[doc(hidden)]
    _Extensible,
}


/// Serial Interface

pub struct Serial<'a, U>(pub &'a U)
where
    U: Any + Usart;

impl<'a, U> Clone for Serial<'a, U>
where
    U: Any + Usart,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U> Copy for Serial<'a, U>
where
    U: Any + Usart,
{
}

impl<'a, U> hal::serial::Read<u8> for Serial<'a, U>
where
    U: Any + Usart,
{
    type Error = Error;

    fn read(&self) -> Result<u8> {
        let uart = self.0;
        let usr = uart.uart_usr.read();

        if usr.receive_fifo_not_empty().bit_is_set() {
            Ok(unsafe {
                ptr::read_volatile::<u8>(&uart.uart_rbr as *const _ as *const u8)
            })
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'a, U> hal::serial::Write<u8> for Serial<'a, U>
where
    U: Any + Usart,
{
    type Error = Error;
    
    fn write(&self, byte: u8) -> Result<()> {
        let uart = self.0;
        let usr = uart.uart_usr.read();

        if usr.trans_fifo_not_full().bit_is_clear() {
            Err(nb::Error::Other(Error::Overrun))
        } else {
            unsafe {
            	// THR is not generated because it overlaps with RBR
                // ptr::write_volatile::<u8>(&uart.uart_thr as *const _ as *mut u8, byte)
                ptr::write_volatile::<u8>(&uart.uart_rbr as *const _ as *mut u8, byte)
            }
            Ok(())
        }
    }
}