#![feature(const_fn)]
#![feature(get_type_id)]
#![feature(never_type)]
#![feature(unsize)]
#![no_std]

extern crate embedded_hal as hal;
extern crate nb;

pub extern crate rk3399_tools;

pub mod serial;
pub mod clock;
pub mod i2c;