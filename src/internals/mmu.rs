use core::arch::asm;

const SECTION_PAGE_TABLE_BASE: u32 = 0xC02;

pub fn initialize() {
    unsafe {
        invalidate_tlb();
        setup_page_tables();
        initialize_ttbcr();
        initialize_ttbr0();
        setup_domains();
        enable_mmu();
    }
}

unsafe fn invalidate_tlb() {
    asm!("mcr p15, 0, r1, c8, c7, 0")
}

unsafe fn setup_page_tables() {
    for i in 0..4096 {
        level1_page_table[i as usize] = SECTION_PAGE_TABLE_BASE | (i << 20);
    }
}

unsafe fn initialize_ttbcr() {
    asm!("mov r1, #0", "mcr p15, 0, r1, c2, c0, 2")
}

unsafe fn initialize_ttbr0() {
    let page_table_addr: u32 = level1_page_table.as_ptr() as u32;

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

extern "C" {
    static mut level1_page_table: [u32; 4096];
}
