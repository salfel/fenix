build:
	rm -rf out
	mkdir out
	arm-none-eabi-gcc -mcpu=cortex-a8 -c src/asm/setup.S -o out/setup.o
	arm-none-eabi-gcc -mcpu=cortex-a8 -c src/asm/interrupts.S -o out/interrupts.o
	arm-none-eabi-gcc -mcpu=cortex-a8 -c src/asm/exceptions.S -o out/exceptions.o
	arm-none-eabi-gcc -mcpu=cortex-a8 -c src/asm/software_interrupts.S -o out/software_interrupts.o
	arm-none-eabi-gcc -mcpu=cortex-a8 -c src/asm/kernel.S -o out/kernel.o
	# arm-none-eabi-ld  -T boot/linker.ld out/setup.o out/interrupt.o out/exceptions.o out/software_interrupt.o  out/kernel.o out/main.o -o out/kernel.elf
	cargo build --release
	cp target/armv7a-none-eabi/release/fenix out/kernel.elf
	arm-none-eabi-objdump -d out/kernel.elf > out/kernel.dump
	arm-none-eabi-objdump -t out/kernel.elf > out/kernel.map
	arm-none-eabi-objcopy out/kernel.elf -O binary out/boot.bin
	cat boot/toc.bin boot/header.bin out/boot.bin > out/rom.img

qemu: 
	qemu-system-arm -M cubieboard -cpu cortex-a8 -kernel out/kernel.elf

flash:
	while ! lsblk | grep -q 'sda'; do sleep 1; done
	sudo dd if=./out/rom.img of=/dev/sda oflag=direct bs=4M status=progress
	sync
