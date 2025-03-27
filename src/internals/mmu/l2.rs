use crate::internals::mmu::l1::{level1_page_table, L1PointerTableEntry};

const L2_FAULT_PAGE_TABLE_ENTRY: u32 = 0x0;

pub fn initialize() {
    let l1_pointer = L1PointerTableEntry::new(&raw mut LEVEL2_PAGE_TABLE);
    for i in 0..1024 {
        unsafe {
            LEVEL2_PAGE_TABLE.0[i as usize] = L2_FAULT_PAGE_TABLE_ENTRY;
        }
    }

    unsafe {
        level1_page_table[0] = l1_pointer.into();
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

struct L2SmallPageTableEntry {
    address: u32,
    permissions: AccessPermissions,
}

impl L2SmallPageTableEntry {
    pub fn new(address: u32) -> Self {
        L2SmallPageTableEntry {
            address,
            permissions: AccessPermissions::Full,
        }
    }
}

impl From<L2SmallPageTableEntry> for u32 {
    fn from(val: L2SmallPageTableEntry) -> Self {
        let L2SmallPageTableEntry {
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
