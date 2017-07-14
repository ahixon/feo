#![feature(const_fn)]
#![feature(get_type_id)]
#![feature(never_type)]
#![feature(unsize)]
#![no_std]

extern crate embedded_hal as hal;
extern crate nb;

#[cfg(target_arch = "aarch64")]
pub extern crate rk3399_tools;

#[cfg(not(target_arch = "aarch64"))]
pub extern crate rk3399_m0;

pub mod serial;
pub mod clock;
pub mod i2c;