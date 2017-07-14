all:
	# convert lilmemcap to flat binary
	arm-none-eabi-objcopy -O binary ../lilmemcap/target/thumbv6m-none-eabi/debug/lilmemcap target/lilmemcap.bin

	# convert to ld linker script
	cat target/lilmemcap.bin | hexdump -v -e '"BYTE(0x" 1/1 "%02X" ")\n"' > target/lilmemcap.ld

	# stick binary into object file
	#ld -r -b binary lilmemcap.bin -o lilmemcap.bin.o
	#objcopy --rename-section .data=.lilmemcap lilmemcap.bin.o

	# build and link ourselves
	xargo build --target aarch64-unknown-linux-gnu

	# copy to tftp
	cp target/aarch64-unknown-linux-gnu/debug/feo /srv/tftp/firefly.elf

	# reset board
	ssh root@tplink-w './reset.sh'
