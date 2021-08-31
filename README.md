Rust code that runs on the AP on the RK3399, initialises and then chainloads [lilmemcap](https://github.com/ahixon/lilmemcap) onto the M0 processor.

It's expected that the image is loaded by TFTP by the built in bootloader. AP bringup is mostly handled by the bootloader right now. Eventually though there's no reason why this couldn't also be loaded directly from SRAM.

# Installation

	curl https://sh.rustup.rs -sSf | sh
	cargo install xargo
	rustup component add rust-src

# Building

To build `feo`:

	xargo build --target aarch64-unknown-linux-gnu

To build everything together:

	make
