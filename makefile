build: 
	rm -rf out
	mkdir out
	arm-none-eabi-as --noexecstack boot/start.asm -o out/start.o
	rustc -C lto --target armv7a-none-eabi -o out/kernel.o -C panic=abort -O --emit=obj src/main.rs
	arm-none-eabi-ld -T boot/linker.ld -o out/boot.elf out/start.o out/kernel.o
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
	while ! lsblk | grep -q 'sda'; do sleep 1; done
	sudo dd if=./out/fenix.img of=/dev/sda oflag=direct bs=4M status=progress
	sync
	sudo partprobe /dev/sda
