use core::arch::asm;

use super::l1::{L1PointerTableEntry, LEVEL1_PAGE_TABLE};

const BASE_ADDRESS: u32 = 0x4030_0000;
const PAGE_SIZE_BITS: u32 = 12;
const PAGE_SIZE: u32 = 0x1000;
const PAGE_TABLE_SIZE: usize = 256;
const L2_FAULT_PAGE_TABLE_ENTRY: u32 = 0x0;

/// Initializes the Level 2 page table and updates the Level 1 pointer table.
/// 
/// This function sets every entry in the Level 2 page table to a fault entry, ensuring that the memory
/// starts in a known fault state. It then creates a new Level 1 pointer table entry referencing the Level 2
/// page table and stores the converted pointer into the first entry of the Level 1 page table. This routine
/// is intended to be called during system initialization.
/// 
/// # Examples
/// 
/// ```
/// // Initialize the page tables. After this call, all Level 2 page table entries are set to the fault entry.
/// initialize();
/// ```
pub fn initialize() {
    let l1_pointer = L1PointerTableEntry::new(&raw mut LEVEL2_PAGE_TABLE);
    for i in 0..PAGE_TABLE_SIZE {
        unsafe {
            LEVEL2_PAGE_TABLE.0[i] = L2_FAULT_PAGE_TABLE_ENTRY;
        }
    }

    unsafe {
        LEVEL1_PAGE_TABLE.0[0] = l1_pointer.into();
    }
}

#[no_mangle]
static mut LEVEL2_PAGE_TABLE: L2PageTable = L2PageTable::new();

#[repr(align(1024))]
pub struct L2PageTable([u32; PAGE_TABLE_SIZE]);

impl L2PageTable {
    /// Creates a new `L2PageTable` with all entries initialized to zero.
    ///
    /// This constant function returns an instance of `L2PageTable` where each entry is set to `0`,
    /// representing a default fault state ready for further setup in the memory management process.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new L2 page table instance.
    /// let table = L2PageTable::new();
    ///
    /// // Verify that all entries are initialized to 0.
    /// assert!(table.0.iter().all(|&entry| entry == 0));
    /// ```
    const fn new() -> Self {
        L2PageTable([0; PAGE_TABLE_SIZE])
    }
}

#[no_mangle]
static mut USED_PAGES: [bool; PAGE_TABLE_SIZE] = [false; PAGE_TABLE_SIZE];

pub struct L2SmallPageTableEntry {
    asid: Option<u32>,
    virtual_address: u32,
    physical_address: u32,
    permissions: AccessPermissions,
}

impl L2SmallPageTableEntry {
    /// Attempts to create a new L2 page table entry using an optional address space identifier (ASID).
    ///
    /// This function scans for the first available page in the memory tracking array. When a free page is found,
    /// it marks the page as used, computes its physical address based on the system's base address and page size,
    /// and returns a new `L2SmallPageTableEntry` initialized with a default virtual address of 0 and full access permissions.
    ///
    /// Returns `Some(entry)` if a free page is available; otherwise, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l2::L2SmallPageTableEntry;
    ///
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(42)) {
    ///     assert_eq!(entry.asid, Some(42));
    ///     // Additional validations (e.g., on physical address or permissions) can be performed here.
    /// } else {
    ///     panic!("Failed to allocate a new page table entry");
    /// }
    /// ```
    pub fn try_new(asid: Option<u32>) -> Option<Self> {
        let current_index =
            (0..PAGE_TABLE_SIZE as u32).find(|&i| unsafe { !USED_PAGES[i as usize] })?;
        unsafe {
            USED_PAGES[current_index as usize] = true;
        }
        let offset = current_index << PAGE_SIZE_BITS;

        Some(L2SmallPageTableEntry {
            asid,
            virtual_address: 0,
            physical_address: BASE_ADDRESS + offset,
            permissions: AccessPermissions::Full,
        })
    }

    /// Returns an empty `L2SmallPageTableEntry` with default field values.
    ///
    /// This constant function creates an instance of `L2SmallPageTableEntry` with no associated ASID,
    /// both virtual and physical addresses set to zero, and full access permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::internals::mmu::l2::{L2SmallPageTableEntry, AccessPermissions}; // Adjust the import path as needed.
    /// let entry = L2SmallPageTableEntry::empty();
    /// assert_eq!(entry.asid, None);
    /// assert_eq!(entry.virtual_address, 0);
    /// assert_eq!(entry.physical_address, 0);
    /// assert_eq!(entry.permissions, AccessPermissions::Full);
    /// ```
    pub const fn empty() -> Self {
        L2SmallPageTableEntry {
            asid: None,
            virtual_address: 0,
            physical_address: 0,
            permissions: AccessPermissions::Full,
        }
    }

    /// Updates the processor's ASID register with this entry's ASID value.
    ///
    /// If the `asid` field is set, the function writes the ASID to the processor's ASID register
    /// using an inline assembly instruction. If the `asid` is not set, no action is performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Example usage of set_asid:
    /// // Construct a sample L2SmallPageTableEntry with an ASID.
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(42),
    ///     virtual_address: 0x1000,
    ///     // Initialize other necessary fields...
    /// };
    /// 
    /// // Update the processor's ASID register with the entry's ASID.
    /// entry.set_asid();
    /// ```
    pub fn set_asid(&self) {
        if let Some(asid) = self.asid {
            unsafe {
                asm!("mcr p15, 0, {asid}, c13, c0, 1", asid = in(reg) asid);
            }
        }
    }

    /// Invalidates the Translation Lookaside Buffer (TLB) entry for the page corresponding to this table entry.
    ///
    /// The method computes a page-aligned address from the stored virtual address and incorporates the ASID (if present)
    /// to construct a hardware-specific invalidation command using inline assembly. This ensures that any stale translation
    /// for the given page is removed from the TLB.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `entry` is a valid L2SmallPageTableEntry instance with a populated `virtual_address` and optional ASID.
    /// entry.invalidate_tlb();
    /// ```
    fn invalidate_tlb(&self) {
        unsafe {
            asm!("mcr p15, 0, {mva}, c8, c7, 1", mva = in(reg) (self.virtual_address & !0xFFF) | self.asid.unwrap_or(0));
        };
    }

    /// Registers this L2 page table entry in the global level 2 page table.
    ///
    /// This method sets the ASID for the entry (if available)
    /// and then writes its 32-bit representation into the `LEVEL2_PAGE_TABLE`.
    /// The appropriate index in the table is computed by shifting the virtual
    /// address right by the number of bits that represent the page size.
    ///
    /// # Safety
    ///
    /// This function performs an unsafe update on a global mutable static variable.
    ///
    /// # Examples
    ///
    /// ```
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(5)) {
    ///     entry.register();
    ///     // The entry is now registered in the global LEVEL2_PAGE_TABLE.
    /// }
    /// ```
    pub fn register(&self) {
        self.set_asid();

        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] = self.into();
        }
    }

    /// Unregisters the page table entry by marking it as a fault and freeing its associated physical page.
    /// 
    /// This method updates the level 2 page table entry corresponding to the entry’s virtual address to a fault value,
    /// resets the used flag for the physical page, and then invalidates the Translation Lookaside Buffer (TLB) for the page.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Attempt to create and register a new page table entry.
    /// let entry = L2SmallPageTableEntry::try_new(Some(42))
    ///     .expect("Failed to create a page table entry");
    /// 
    /// // When the page is no longer needed, unregister the entry.
    /// entry.unregister();
    /// ```
    pub fn unregister(&self) {
        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] =
                L2_FAULT_PAGE_TABLE_ENTRY;
            USED_PAGES[(self.physical_address - BASE_ADDRESS) as usize >> PAGE_SIZE_BITS] = false;
        }

        self.invalidate_tlb();
    }

    /// Returns the starting virtual address of this L2 page table entry.
    ///
    /// This method retrieves the virtual address associated with the entry, marking the
    /// beginning of the virtual memory region managed by it.
    ///
    /// # Examples
    ///
    /// ```
    /// // Attempt to create a new L2SmallPageTableEntry with an optional ASID.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(1)) {
    ///     let start_addr = entry.start();
    ///     assert_eq!(start_addr, entry.virtual_address);
    /// }
    /// ```
    pub fn start(&self) -> u32 {
        self.virtual_address
    }

    /// Returns the ending virtual address of the page range covered by the entry.
    ///
    /// The ending address is calculated by adding the system-defined page size to the entry's starting virtual address
    /// and then subtracting 4 bytes, which accounts for the fixed offset of the page table entry size.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create an empty L2SmallPageTableEntry and assign a starting virtual address.
    /// let mut entry = L2SmallPageTableEntry::empty();
    /// entry.virtual_address = 0x1000;
    /// // PAGE_SIZE should be defined in scope; for example, if PAGE_SIZE is 4096:
    /// assert_eq!(entry.end(), 0x1000 + PAGE_SIZE - 4);
    /// ```
    pub fn end(&self) -> u32 {
        self.virtual_address + PAGE_SIZE - 4
    }
}

impl From<&L2SmallPageTableEntry> for u32 {
    /// Converts a reference to a `L2SmallPageTableEntry` into its 32-bit representation.
    ///
    /// The returned value encodes the entry's physical address along with a non-global flag
    /// (set if an ASID is present) and its access permissions. A constant flag (0b10) is also included
    /// to mark the entry as valid.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example setup for a page table entry.
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(42),
    ///     virtual_address: 0x1000,
    ///     physical_address: 0x2000,
    ///     permissions: AccessPermissions::FullAccess,
    /// };
    ///
    /// // Convert the entry to a 32-bit representation.
    /// let encoded: u32 = u32::from(&entry);
    ///
    /// // Verify that the constant flag (0b10) is set.
    /// assert_eq!(encoded & 0b11, 0b10);
    /// ```
    fn from(val: &L2SmallPageTableEntry) -> Self {
        let L2SmallPageTableEntry {
            asid,
            virtual_address: _,
            physical_address: address,
            permissions,
        } = val;
        let permissions: u32 = permissions.into();
        let non_global = asid.is_some() as u32;

        address | non_global << 11 | permissions | 0b10
    }
}

enum AccessPermissions {
    Full,
}

impl From<&AccessPermissions> for u32 {
    /// Converts a reference to an `AccessPermissions` variant into its corresponding `u32` encoding.
    /// 
    /// The conversion encodes "full" access permissions by shifting the bit pattern left by 4 bits.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use crate::AccessPermissions;
    ///
    /// let permission = AccessPermissions::Full;
    /// let encoded: u32 = (&permission).into();
    /// assert_eq!(encoded, 0b11 << 4);
    /// ```
    fn from(value: &AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 4,
        }
    }
}
