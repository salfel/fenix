use core::arch::asm;

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

enum PageTableEntry {
    Section(SectionPageTableEntry),
}

impl From<PageTableEntry> for u32 {
    fn from(value: PageTableEntry) -> Self {
        match value {
            PageTableEntry::Section(section) => section.into(),
        }
    }
}

struct SectionPageTableEntry {
    index: u32,
    access_permissions: AccessPermissions,
}

impl From<SectionPageTableEntry> for u32 {
    fn from(val: SectionPageTableEntry) -> Self {
        let SectionPageTableEntry {
            index,
            access_permissions,
        } = val;
        {
            let permissions: u32 = access_permissions.into();
            (index << 20) | permissions | 0b10
        }
    }
}

enum AccessPermissions {
    Full,
}

impl From<AccessPermissions> for u32 {
    fn from(val: AccessPermissions) -> Self {
        match val {
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}

unsafe fn invalidate_tlb() {
    asm!("mcr p15, 0, r1, c8, c7, 0")
}

unsafe fn setup_page_tables() {
    for i in 0..4096 {
        let section = PageTableEntry::Section(SectionPageTableEntry {
            index: i,
            access_permissions: AccessPermissions::Full,
        });
        level1_page_table[i as usize] = section.into();
    }
}

unsafe fn initialize_ttbcr() {
    asm!("mov r1, #0", "mcr p15, 0, r1, c2, c0, 2")
}

unsafe fn initialize_ttbr0() {
    let page_table_addr: u32 = &raw const level1_page_table as u32;

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
