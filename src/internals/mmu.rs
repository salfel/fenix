use core::arch::asm;

pub fn initialize() {
    unsafe {
        setup_page_tables();
        setup_level2_page_table();
        initialize_ttbcr();
        initialize_ttbr0();
        invalidate_tlb();
        setup_domains();
        enable_mmu();
    }
}

enum L1PageTableEntry {
    Fault,
    Section(L1SectionPageTableEntry),
    Pointer(L1PointerTableEntry),
}

impl L1PageTableEntry {
    pub fn new_section(index: u32) -> Self {
        L1PageTableEntry::Section(L1SectionPageTableEntry {
            index,
            access_permissions: AccessPermissions::Full,
        })
    }
}

impl From<L1PageTableEntry> for u32 {
    fn from(value: L1PageTableEntry) -> Self {
        match value {
            L1PageTableEntry::Fault => 0x0,
            L1PageTableEntry::Section(section) => section.into(),
            L1PageTableEntry::Pointer(pointer) => pointer.into(),
        }
    }
}

struct L1SectionPageTableEntry {
    index: u32,
    access_permissions: AccessPermissions,
}

impl From<L1SectionPageTableEntry> for u32 {
    fn from(val: L1SectionPageTableEntry) -> Self {
        let L1SectionPageTableEntry {
            index,
            access_permissions,
        } = val;

        let permissions: u32 = match access_permissions {
            AccessPermissions::Full => 0b11 << 10,
        };
        (index << 20) | permissions | 0b10
    }
}

struct L1PointerTableEntry {
    table: *mut L2PageTable,
}

impl L1PointerTableEntry {
    pub fn new(table: *mut L2PageTable) -> Self {
        L1PointerTableEntry { table }
    }
}

impl From<L1PointerTableEntry> for u32 {
    fn from(val: L1PointerTableEntry) -> Self {
        let L1PointerTableEntry { table } = val;
        table as u32 | 0b01
    }
}

enum AccessPermissions {
    Full,
}

#[repr(align(1024))]
struct L2PageTable([u32; 1024]);

impl L2PageTable {
    const fn new() -> Self {
        L2PageTable([0; 1024])
    }
}

struct L2PageTableEntry {
    address: u32,
    permissions: AccessPermissions,
}

impl L2PageTableEntry {
    pub fn new(address: u32) -> Self {
        L2PageTableEntry {
            address,
            permissions: AccessPermissions::Full,
        }
    }
}

impl From<L2PageTableEntry> for u32 {
    fn from(val: L2PageTableEntry) -> Self {
        let L2PageTableEntry {
            address,
            permissions,
        } = val;
        let permissions: u32 = match permissions {
            AccessPermissions::Full => 0b11 << 4,
        };
        address | permissions | 0b10
    }
}

#[no_mangle]
static mut LEVEL2_PAGE_TABLE: L2PageTable = L2PageTable::new();

unsafe fn invalidate_tlb() {
    asm!("mcr p15, 0, r1, c8, c7, 0")
}

unsafe fn setup_page_tables() {
    for i in 0..4096 {
        let section = L1PageTableEntry::new_section(i);
        level1_page_table[i as usize] = section.into();
    }
}

unsafe fn setup_level2_page_table() {
    const BASE_ADDRESS: u32 = 0x40300000;
    let l1_pointer = L1PointerTableEntry::new(&raw mut LEVEL2_PAGE_TABLE);
    for i in 0..1024 {
        let entry = L2PageTableEntry::new(BASE_ADDRESS | i << 12);
        LEVEL2_PAGE_TABLE.0[i as usize] = entry.into();
    }

    level1_page_table[BASE_ADDRESS as usize >> 20] = L1PageTableEntry::Pointer(l1_pointer).into();
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
