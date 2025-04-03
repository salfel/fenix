use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=src/asm/exceptions.S");
    println!("cargo:rerun-if-changed=src/asm/setup.S");
    println!("cargo:rerun-if-changed=src/asm/kernel.S");
    println!("cargo:rerun-if-changed=src/asm/software_interrupts.S");
    println!("cargo:rerun-if-changed=src/asm/interrupts.S");
    println!("cargo:rerun-if-changed=src/asm/mmu.S");
    println!("cargo:rerun-if-changed=boot/linker.ld");

    println!("cargo:rerun-if-changed=programs");

    for entry in fs::read_dir("programs").unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            println!("cargo:rerun-if-changed=programs/{}", path.display());
        }
    }
}
