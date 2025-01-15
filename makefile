build:
	mkdir -p out
	# arm-none-eabi-gcc -c _start.S -o start.o
	cargo build --release
	arm-none-eabi-objcopy -O binary target/armv7a-none-eabi/release/fenix out/boot.bin
	cat boot/toc.bin boot/header.bin out/boot.bin > out/fenix.img
