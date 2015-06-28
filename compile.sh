#!/bin/sh

cargo build --target armv5te-none-eabi --verbose --release

arm-none-eabi-size target/armv5te-none-eabi/release/arietta-ymodem-boot
arm-none-eabi-objcopy -O binary target/armv5te-none-eabi/release/arietta-ymodem-boot kernel.bin
mkimage -A arm -O linux -T standalone -C none -a 23000000 -e 23000000 -n kernel -d kernel.bin uImage
