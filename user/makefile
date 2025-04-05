build:
	rm -rf out
	mkdir out
	cargo build --release
	cp ../target/armv7a-none-eabi/release/user out/kernel.elf
	arm-none-eabi-objdump -d out/kernel.elf > out/kernel.dump
	arm-none-eabi-nm out/kernel.elf > out/kernel.map
	arm-none-eabi-objcopy out/kernel.elf -O binary out/kernel.bin
