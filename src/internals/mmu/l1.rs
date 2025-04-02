use core::ops::Range;

use super::l2::L2PageTable;

const PAGE_SIZE: u32 = 1 << 20;
const PAGE_SIZE_BITS: u32 = 20;
const PAGE_TABLE_SIZE: usize = 4096;

pub fn initialize() {
    let peripheral_memory: Range<u32> = 0x4400_0000..0x8000_0000;
    let kernel_memory: Range<u32> = 0x4020_0000..0x4040_0000;

    enable_memory_range(kernel_memory, AccessPermissions::UserReadOnly);
    enable_memory_range(peripheral_memory, AccessPermissions::Full);
}

fn enable_memory_range(range: Range<u32>, permissions: AccessPermissions) {
    for page in range.step_by(PAGE_SIZE as usize) {
        let section = L1SectionPageTableEntry::new(page, permissions);
        unsafe {
            LEVEL1_PAGE_TABLE.0[page as usize >> PAGE_SIZE_BITS] = section.into();
        }
    }
}

pub static mut LEVEL1_PAGE_TABLE: L1PageTable = L1PageTable::new();

#[repr(align(16384))]
pub struct L1PageTable(pub [u32; PAGE_TABLE_SIZE]);

impl L1PageTable {
    const fn new() -> Self {
        L1PageTable([0; PAGE_TABLE_SIZE])
    }
}

pub struct L1SectionPageTableEntry {
    address: u32,
    access_permissions: AccessPermissions,
}

impl L1SectionPageTableEntry {
    fn new(address: u32, access_permissions: AccessPermissions) -> Self {
        L1SectionPageTableEntry {
            address,
            access_permissions,
        }
    }
}

impl From<L1SectionPageTableEntry> for u32 {
    fn from(val: L1SectionPageTableEntry) -> Self {
        let L1SectionPageTableEntry {
            address,
            access_permissions,
        } = val;

        let permissions: u32 = access_permissions.into();
        address | permissions | 0b10
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

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum AccessPermissions {
    Privileged,
    UserReadOnly,
    Full,
}

impl From<AccessPermissions> for u32 {
    fn from(value: AccessPermissions) -> Self {
        match value {
            AccessPermissions::UserReadOnly => 0b10 << 10,
            AccessPermissions::Privileged => 0b01 << 10,
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}
