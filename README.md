Currently just the Rust code to semi-boot and then run on the AP on the RK3399, and chainloads [lilmemcap](https://github.com/ahixon/lilmemcap) onto the M0 processor.

# Installation

	curl https://sh.rustup.rs -sSf | sh
	cargo install xargo
	rustup component add rust-src

# Building

	xargo build --target aarch64-unknown-linux-gnu
