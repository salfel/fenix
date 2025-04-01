use core::arch::asm;

use super::l1::{L1PointerTableEntry, LEVEL1_PAGE_TABLE};

const BASE_ADDRESS: u32 = 0x4030_0000;
const PAGE_SIZE_BITS: u32 = 12;
const PAGE_SIZE: u32 = 0x1000;
const PAGE_TABLE_SIZE: usize = 256;
const L2_FAULT_PAGE_TABLE_ENTRY: u32 = 0x0;

/// Initializes the L2 page table and registers its pointer in the Level 1 table.
///
/// This function resets every entry in the L2 page table to the fault entry and creates
/// a new Level 1 pointer table entry associated with the global Level 2 page table. The
/// pointer is then stored as the first entry in the Level 1 page table. This setup is
/// essential for establishing the initial memory mapping during system initialization.
///
/// # Examples
///
/// ```
/// // Initialize the memory management system's L2 page table.
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
    /// Creates a new L2 page table with all entries initialized to zero.
    ///
    /// This constant function returns an instance of `L2PageTable` with an internal array of size
    /// `PAGE_TABLE_SIZE`, where every element is set to 0, indicating that no pages are mapped.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new L2PageTable instance.
    /// const TABLE: L2PageTable = L2PageTable::new();
    /// // Verify that all entries are initialized to 0.
    /// assert!(TABLE.0.iter().all(|&entry| entry == 0));
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
    /// Attempts to create a new L2 small page table entry with an optional ASID.
    ///
    /// This function searches for the first available page in the L2 page table. When an unused page is found,
    /// it marks that page as used, computes the physical address based on the page's index and a predefined base address,
    /// and creates an entry with full access permissions. If no free page is available, the function returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Attempt to create a new page table entry with an optional ASID of 1.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(1)) {
    ///     assert_eq!(entry.asid, Some(1));
    ///     assert!(entry.physical_address >= BASE_ADDRESS);
    /// } else {
    ///     panic!("No available page found");
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

    /// Returns an empty L2SmallPageTableEntry with default values.
    ///
    /// This function creates a page table entry with:
    /// - `asid` set to `None`,
    /// - `virtual_address` set to `0`,
    /// - `physical_address` set to `0`, and
    /// - `permissions` set to `AccessPermissions::Full`.
    ///
    /// # Examples
    ///
    /// ```
    /// let entry = L2SmallPageTableEntry::empty();
    /// assert!(entry.asid.is_none());
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

    /// Sets the Address Space Identifier (ASID) for this page table entry.
    ///
    /// If an ASID is present, this method updates the processor’s system register using an inline assembly
    /// instruction. If no ASID is set (i.e., `self.asid` is `None`), the method performs no action.
    ///
    /// # Examples
    ///
    /// ```
    /// // Attempt to create a new page table entry with a valid ASID.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(0x42)) {
    ///     entry.set_asid();
    /// }
    /// ```
    pub fn set_asid(&self) {
        if let Some(asid) = self.asid {
            unsafe {
                asm!("mcr p15, 0, {asid}, c13, c0, 1", asid = in(reg) asid);
            }
        }
    }

    /// Invalidates the TLB entry corresponding to this page table entry's virtual address.
    ///
    /// The function computes the page-aligned virtual address and combines it with the ASID (if available)
    /// before issuing an ARM system instruction via inline assembly to purge the stale TLB mapping.
    /// This ensures that subsequent memory accesses trigger an updated lookup for the page.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume `L2SmallPageTableEntry` is properly constructed with an ASID and a virtual address.
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(42),
    ///     virtual_address: 0x2000,
    ///     // Initialize other fields as necessary.
    /// };
    /// entry.invalidate_tlb();
    /// ```
    fn invalidate_tlb(&self) {
        unsafe {
            asm!("mcr p15, 0, {mva}, c8, c7, 1", mva = in(reg) (self.virtual_address & !0xFFF) | self.asid.unwrap_or(0));
        };
    }

    /// Registers this L2 page table entry into the global level 2 page table.
    ///
    /// This method first updates the entry's address space identifier (ASID) by calling `set_asid()`.
    /// It then writes a converted representation of the entry into the appropriate slot in the level 2 page table,
    /// where the index is determined by shifting the entry's virtual address right by `PAGE_SIZE_BITS`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new L2SmallPageTableEntry (assuming the try_new method returns a valid entry)
    /// let entry = L2SmallPageTableEntry::try_new(Some(1))
    ///     .expect("Failed to create page table entry");
    /// entry.register();
    /// // The entry is now registered in the global level 2 page table.
    /// ```
    pub fn register(&self) {
        self.set_asid();

        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] = self.into();
        }
    }

    /// Unregisters the L2 page table entry.
    ///
    /// This method clears the corresponding entry in the global L2 page table by setting it to the fault entry,
    /// marks the associated physical page as free, and then invalidates the Translation Lookaside Buffer (TLB)
    /// for the entry's virtual address.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new L2 page table entry with an ASID.
    /// let entry = L2SmallPageTableEntry::try_new(Some(1)).expect("Failed to create entry");
    ///
    /// // Register the entry to establish it in the page table.
    /// entry.register();
    ///
    /// // Unregister the entry when it is no longer needed.
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

    /// Returns the starting virtual address of this page table entry.
    ///
    /// This method retrieves the virtual address that marks the beginning of the memory range associated with the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct an example page table entry with a predefined virtual address.
    /// let entry = L2SmallPageTableEntry {
    ///     virtual_address: 0x1000,
    ///     asid: Some(1),
    ///     // Other fields are initialized as needed.
    /// };
    ///
    /// assert_eq!(entry.start(), 0x1000);
    /// ```
    pub fn start(&self) -> u32 {
        self.virtual_address
    }

    /// Returns the ending virtual address of the memory region for this page table entry.
    ///
    /// The result is computed by adding the page size to the entry’s starting virtual address and subtracting 4,
    /// yielding the final valid address within the allocated page.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l2::{L2SmallPageTableEntry, PAGE_SIZE};
    ///
    /// // Create a new page table entry with an ASID; `try_new` returns an Option.
    /// let entry = L2SmallPageTableEntry::try_new(Some(42)).expect("Failed to create page table entry");
    ///
    /// // The expected end address is the starting address plus PAGE_SIZE minus 4.
    /// let expected_end = entry.virtual_address + PAGE_SIZE - 4;
    /// assert_eq!(entry.end(), expected_end);
    /// ```
    pub fn end(&self) -> u32 {
        self.virtual_address + PAGE_SIZE - 4
    }
}

impl From<&L2SmallPageTableEntry> for u32 {
    /// Converts a reference to an `L2SmallPageTableEntry` into a 32-bit page table entry.
    ///
    /// This function encodes the entry's physical address, a flag indicating the presence of an ASID
    /// (set as the non-global bit at bit 11), and the access permissions. It also sets a constant flag (0b10)
    /// in the resulting value.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example conversion of an L2SmallPageTableEntry into its u32 representation.
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(42),
    ///     virtual_address: 0x1000,
    ///     physical_address: 0x2000,
    ///     permissions: AccessPermissions::FullAccess,
    /// };
    /// let encoded: u32 = u32::from(&entry);
    /// // Confirm that the constant flag (0b10) is present in the encoded value.
    /// assert_eq!(encoded & 0b10, 0b10);
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
    /// Converts a reference to an `AccessPermissions` into its corresponding 32-bit bitfield representation.
    ///
    /// This function maps [`AccessPermissions::Full`] to a bit pattern where the two most significant bits in the lower nibble are set (`0b11 << 4`).
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming AccessPermissions is defined with a variant Full:
    /// enum AccessPermissions {
    ///     Full,
    /// }
    ///
    /// impl From<&AccessPermissions> for u32 {
    ///     fn from(value: &AccessPermissions) -> Self {
    ///         match value {
    ///             AccessPermissions::Full => 0b11 << 4,
    ///         }
    ///     }
    /// }
    ///
    /// let permission = AccessPermissions::Full;
    /// let bits = u32::from(&permission);
    /// assert_eq!(bits, 0b11 << 4);
    /// ```
    fn from(value: &AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 4,
        }
    }
}
