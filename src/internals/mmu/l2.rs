use crate::internals::mmu::l1::{level1_page_table, L1PageTableEntry, L1PointerTableEntry};

pub fn initialize() {
    const BASE_ADDRESS: u32 = 0x40300000;
    let l1_pointer = L1PointerTableEntry::new(&raw mut LEVEL2_PAGE_TABLE);
    for i in 0..1024 {
        let entry = L2PageTableEntry::new(BASE_ADDRESS | i << 12);
        unsafe {
            LEVEL2_PAGE_TABLE.0[i as usize] = entry.into();
        }
    }

    unsafe {
        level1_page_table[0] = L1PageTableEntry::Pointer(l1_pointer).into();
    }
}

#[no_mangle]
static mut LEVEL2_PAGE_TABLE: L2PageTable = L2PageTable::new();

#[repr(align(1024))]
pub struct L2PageTable([u32; 1024]);

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
        let permissions: u32 = permissions.into();
        address | permissions | 0b10
    }
}

enum AccessPermissions {
    Full,
}

impl From<AccessPermissions> for u32 {
    fn from(value: AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}
