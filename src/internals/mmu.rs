use core::arch::asm;

#[no_mangle]
pub fn initialize() {
    unsafe {
        disable_mmu();
        setup_l1_cache();
        invalidate_data_cache();
        invalidate_tlb();
        branch_prediction_enable();
        d_side_prefetch_enable();
        setup_tables();
        initialize_mmu();
        setup_domains();
        enable_mmu();
    }
}

unsafe fn disable_mmu() {
    asm!(
        "mrc p15, 0, r1, c1, c0, 0",
        "bic r1, r1, #0x1",
        "mcr p15, 0, r1, c1, c0, 0"
    )
}

unsafe fn setup_l1_cache() {
    asm!(
        // disable l1 cache
        "mrc p15, 0, r1, c1, c0, 0",
        "bic r1, r1, #(0x1 << 12)",
        "bic r1, r1, #(0x1 << 2)",
        // invalidate l1 cache
        "mov r1, #0",
        "mcr p15, 0, r1, c7, c5, 0"
    )
}

unsafe fn invalidate_data_cache() {
    let cache_size: u32;

    asm!(
        "mrc p15, 1, r0, c0, c0, 0",
        "ldr r3, =0x1ff",
        "and {0}, r3, r0, lsr #13",
        out(reg) cache_size
    );

    for way in 0..4 {
        for set in 0..cache_size {
            let entry = way << 30 | set << 5;
            asm!("mcr p15, 0, {0}, c7, c6, 2", in(reg) entry);
        }
    }
}

unsafe fn invalidate_tlb() {
    asm!("mcr p15, 0, r1, c8, c7, 0");
}

unsafe fn branch_prediction_enable() {
    asm!(
        "mov r1, #0",
        "mrc p15, 0, r1, c1, c0, 0",
        "orr r1, r1, #(0x1 << 11)",
        "mcr p15, 0, r1, c1, c0, 0"
    )
}

unsafe fn d_side_prefetch_enable() {
    asm!(
        "mrc p15, 0, r1, c1, c0, 1",
        "orr r1, r1, #(0x1 << 2)",
        "mcr p15, 0, r1, c1, c0, 1",
        "dsb",
        "isb"
    );
}

unsafe fn setup_tables() {
    let base = 0b110111100010;
    for i in 0..4096 {
        level1_page_table[i as usize] = base | (i << 20);
    }
}

unsafe fn initialize_mmu() {
    let page_table_addr = &raw const level1_page_table as u32;
    asm!(
        "mov r1, #0",
        "mcr p15, 0, r1, c2, c0, 2",
        "mcr p15, 0, {0}, c2, c0, 0",
        in(reg) page_table_addr
    )
}

unsafe fn setup_domains() {
    asm!("mcr p15, 0, {0}, c3, c0, 0", in(reg) 0x55555555);
}

unsafe fn enable_mmu() {
    let control_reg: u32;
    asm!("mrc p15, 0, {0}, c1, c0, 0", out(reg) control_reg);

    asm!("mcr p15, 0, {0}, c1, c0, 0", in(reg) control_reg | 0x1)
}

extern "C" {
    static mut level1_page_table: [u32; 4096];
}
