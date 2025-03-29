use super::l2::L2PageTable;

pub fn initialize() {
    for i in 0..4096 {
        let section = L1SectionPageTableEntry::new(i);
        unsafe {
            LEVEL1_PAGE_TABLE.0[i as usize] = section.into();
        }
    }
}

pub static mut LEVEL1_PAGE_TABLE: L1PageTable = L1PageTable::new();

#[repr(align(16384))]
pub struct L1PageTable(pub [u32; 4096]);

impl L1PageTable {
    const fn new() -> Self {
        L1PageTable([0; 4096])
    }
}

pub struct L1SectionPageTableEntry {
    index: u32,
    access_permissions: AccessPermissions,
}

impl L1SectionPageTableEntry {
    fn new(index: u32) -> Self {
        L1SectionPageTableEntry {
            index,
            access_permissions: AccessPermissions::Full,
        }
    }
}

impl From<L1SectionPageTableEntry> for u32 {
    fn from(val: L1SectionPageTableEntry) -> Self {
        let L1SectionPageTableEntry {
            index,
            access_permissions,
        } = val;

        let permissions: u32 = access_permissions.into();
        (index << 20) | permissions | 0b10
    }
}

pub struct L1PointerTableEntry {
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

impl From<AccessPermissions> for u32 {
    fn from(value: AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}
