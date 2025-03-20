fn main() {
    println!("cargo:rerun-if-changed=src/asm/exceptions.S");
    println!("cargo:rerun-if-changed=src/asm/setup.S");
    println!("cargo:rerun-if-changed=src/asm/kernel.S");
    println!("cargo:rerun-if-changed=src/asm/software_interrupts.S");
    println!("cargo:rerun-if-changed=src/asm/interrupts.S");
    println!("cargo:rerun-if-changed=boot/linker.ld");
}
