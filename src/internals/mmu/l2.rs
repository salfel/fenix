use core::ops::Range;

use super::{
    l1::{L1PointerTableEntry, LEVEL1_PAGE_TABLE},
    setup::invalidate_tlb,
};

const BASE_ADDRESS: u32 = 0x4030_0000;
const PAGE_SIZE_BITS: u32 = 12;
const PAGE_SIZE: u32 = 0x1000;
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

pub fn register_page() -> Option<L2SmallPageTableEntry> {
    let page = L2SmallPageTableEntry::try_new()?;

    unsafe {
        LEVEL2_PAGE_TABLE.0[page.virtual_address as usize >> PAGE_SIZE_BITS] = (&page).into();
    }

    invalidate_tlb();

    Some(page)
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

pub struct L2SmallPageTableEntry {
    virtual_address: u32,
    physical_address: u32,
    permissions: AccessPermissions,
}

impl L2SmallPageTableEntry {
    pub fn try_new() -> Option<Self> {
        let current_index = first_unused_page()?;
        let offset = current_index << PAGE_SIZE_BITS;

        Some(L2SmallPageTableEntry {
            virtual_address: offset,
            physical_address: BASE_ADDRESS + offset,
            permissions: AccessPermissions::Full,
        })
    }

    pub const fn emptry() -> Self {
        L2SmallPageTableEntry {
            virtual_address: 0,
            physical_address: 0,
            permissions: AccessPermissions::Full
        }
    }

    pub fn register(&self) {
        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] = self.into();
        }

        invalidate_tlb();
    }

    pub fn unregister(&self) {
        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] = L2_FAULT_PAGE_TABLE_ENTRY;
        }

        invalidate_tlb();
    }

    pub fn start(&self) -> u32 {
        self.virtual_address
    }

    pub fn end(&self) -> u32 {
        self.virtual_address + PAGE_SIZE - 4
    }
}

impl From<&L2SmallPageTableEntry> for u32 {
    fn from(val: &L2SmallPageTableEntry) -> Self {
        let L2SmallPageTableEntry {
            virtual_address: _,
            physical_address: address,
            permissions,
        } = val;
        let permissions: u32 = permissions.into();
        address | permissions | 0b10
    }
}

enum AccessPermissions {
    Full,
}

impl From<&AccessPermissions> for u32 {
    fn from(value: &AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 4,
        }
    }
}
