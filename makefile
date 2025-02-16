build:
	rustc -C lto --target arm-unknown-linux-gnueabihf -C panic=abort -o out/kernel.o -O --emit=obj src/main.rs
	arm-none-eabi-gcc -c boot/start.S -o out/start.o
	arm-none-eabi-ld  -T boot/linker.ld out/start.o out/kernel.o -o out/kernel.elf
	arm-none-eabi-objcopy out/kernel.elf -O binary out/boot.bin
	cat boot/toc.bin boot/header.bin out/boot.bin > out/rom.img

flash:
	while ! lsblk | grep -q 'sda'; do sleep 1; done
	sudo dd if=./out/rom.img of=/dev/sda oflag=direct bs=4M status=progress
	sync
