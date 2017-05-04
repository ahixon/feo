#!/bin/sh

TARGET=arm-unknown-linux-gnueabihf

# ensure you have uboot-tools installed

# add -C gzip for gzip'd image
# may also need arm rather than arm64 if loader doesn't like it?
# source: https://www.denx.de/wiki/view/DULG/HowCanICreateAnUImageFromAELFFile
mkimage -A arm64 -O linux -T kernel -a 0x0 -e 0x0 -n feo -d feo-uimage.bin target/debug/$TARGET/feo
/mnt/vodka/private/alex/memento/src/rockchip/rkbin/tools/kernelimage --pack --kernel feo-uimage.bin feo-uimage-with-rk-header.bin