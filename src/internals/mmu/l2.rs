use core::ops::Range;

use super::{
    l1::{L1PointerTableEntry, LEVEL1_PAGE_TABLE},
    setup::invalidate_tlb,
};

const BASE_ADDRESS: u32 = 0x4030_0000;
const PAGE_SIZE_BITS: u32 = 12;
const L2_FAULT_PAGE_TABLE_ENTRY: u32 = 0x0;

pub fn initialize() {
    let l1_pointer = L1PointerTableEntry::new(&raw mut LEVEL2_PAGE_TABLE);
    for i in 0..1024 {
        unsafe {
            LEVEL2_PAGE_TABLE.0[i as usize] = L2_FAULT_PAGE_TABLE_ENTRY;
        }
    }

    unsafe {
        LEVEL1_PAGE_TABLE.0[0] = l1_pointer.into();
    }
}

pub fn register_page() -> Option<Range<u32>> {
    let current_index = match first_unused_page() {
        Some(index) => index,
        None => return None,
    };
    let offset = current_index << PAGE_SIZE_BITS;
    let page = L2SmallPageTableEntry::new(BASE_ADDRESS + offset);

    unsafe {
        LEVEL2_PAGE_TABLE.0[current_index as usize] = page.into();
    }

    invalidate_tlb();

    Some(offset..offset + (1 << PAGE_SIZE_BITS))
}

pub fn unregister_page(page: &Range<u32>) {
    unsafe {
        LEVEL2_PAGE_TABLE.0[page.start as usize >> PAGE_SIZE_BITS] = L2_FAULT_PAGE_TABLE_ENTRY;
    }

    invalidate_tlb();
}

fn first_unused_page() -> Option<u32> {
    (0..1024).find(|&i| unsafe { LEVEL2_PAGE_TABLE.0[i as usize] == 0 })
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
    fn new(address: u32) -> Self {
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
            AccessPermissions::Full => 0b11 << 4,
        }
    }
}
