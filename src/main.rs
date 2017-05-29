#![feature(lang_items)]
#![feature(asm)]
#![feature(unique)]
#![feature(const_fn)]
#![feature(linkage)]
#![feature(compiler_builtins_lib)]

#![no_std]

#[macro_use]
extern crate bitflags;
extern crate spin;
extern crate compiler_builtins;

#[macro_use]
mod serial;
mod lang_items;

// mod m0;
// use m0::{PerilpM0, M0};

fn main() {
	// print!(serial, "\x1b[20h");

	println!("hey babe; going to divide by zero\n");

	// start the M0
	// let addr:u32 = 0x250000;
	// println!("Starting M0 at 0x{:x}...", addr);
	// let mut littleguy = PerilpM0 {serial: serial};
	// unsafe {
		// littleguy.setup (mcuProgramStart as u32);
		// littleguy.setup (addr);
		// littleguy.on ();
	// }
}
