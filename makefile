build:
	rm -rf out
	mkdir out
	cargo build --release
	cp target/armv7a-none-eabi/release/fenix out/boot.elf
	arm-none-eabi-objcopy -O binary out/boot.elf out/boot.bin
	cat boot/toc.bin boot/header.bin out/boot.bin > out/fenix.img

qemu: 
	qemu-system-arm -M cubieboard -cpu cortex-a8 -kernel out/boot.elf

test:
	rm -rf out
	mkdir out
	arm-none-eabi-as boot/test.asm -o out/boot.o
	arm-none-eabi-ld -T boot/linker.ld out/boot.o -o out/boot.elf
	arm-none-eabi-objcopy out/boot.elf -O binary out/boot.bin
	cat boot/toc.bin boot/header.bin out/boot.bin > out/fenix.img
	rm -rf out/boot.o out/boot.elf out/boot.bin

flash:
	sudo dd if=./out/fenix.img of=/dev/sda oflag=direct bs=4M status=progress
	sync
	sudo partprobe /dev/sda
	udisksctl power-off -b /dev/sda
