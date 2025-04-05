use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=kernel/src/asm/exceptions.S");
    println!("cargo:rerun-if-changed=kernel/src/asm/setup.S");
    println!("cargo:rerun-if-changed=kernel/src/asm/kernel.S");
    println!("cargo:rerun-if-changed=kernel/src/asm/software_interrupts.S");
    println!("cargo:rerun-if-changed=kernel/src/asm/interrupts.S");
    println!("cargo:rerun-if-changed=kernel/src/asm/mmu.S");
    println!("cargo:rerun-if-changed=kernel/boot/linker.ld");

    println!("cargo:rerun-if-changed=programs");

    for entry in fs::read_dir("programs").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            println!("cargo:rerun-if-changed=programs/{}", path.display());
        }
    }
}
