use core::arch::asm;

use super::l1::LEVEL1_PAGE_TABLE;

use super::{l1, l2};

pub fn initialize() {
    unsafe {
        l1::initialize();
        l2::initialize();
        initialize_ttbcr();
        initialize_ttbr0();
        invalidate_tlb();
        setup_domains();
        enable_mmu();
    }
}

unsafe fn invalidate_tlb() {
    asm!("mcr p15, 0, r1, c8, c7, 0")
}

unsafe fn initialize_ttbcr() {
    asm!("mov r1, #0", "mcr p15, 0, r1, c2, c0, 2")
}

unsafe fn initialize_ttbr0() {
    let page_table_addr: u32 = &raw const LEVEL1_PAGE_TABLE.0 as u32;

    asm!("mcr p15, 0, {0}, c2, c0, 0", in(reg) page_table_addr);
}

unsafe fn setup_domains() {
    asm!("mcr p15, 0, {0}, c3, c0, 0", in(reg) 0x55555555);
}

unsafe fn enable_mmu() {
    let value: u32;
    asm!("mrc p15, 0, {0}, c1, c0, 0", out(reg) value);
    asm!("mcr p15, 0, {0}, c1, c0, 0", in(reg) value | 0x1);
}
