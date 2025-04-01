use core::arch::asm;

use super::l1::{L1PointerTableEntry, LEVEL1_PAGE_TABLE};

const BASE_ADDRESS: u32 = 0x4030_0000;
const PAGE_SIZE_BITS: u32 = 12;
const PAGE_SIZE: u32 = 0x1000;
const PAGE_TABLE_SIZE: usize = 256;
const L2_FAULT_PAGE_TABLE_ENTRY: u32 = 0x0;

/// Initializes the Level 2 page table by setting all its entries to the fault state and registering its pointer in the Level 1 pointer table.
/// 
/// This function creates a new Level 1 pointer entry associated with the global Level 2 page table. It then iterates over all entries of the Level 2 page table,
/// setting each one to a constant fault entry value (`L2_FAULT_PAGE_TABLE_ENTRY`). Finally, it registers the new pointer in the first slot of the Level 1 pointer table.
/// 
/// # Examples
///
/// ```
/// // Initialize the L2 page table prior to configuring memory mappings.
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
    /// Constructs a new L2PageTable with all entries initialized to zero.
    ///
    /// This constant function returns an L2PageTable with its underlying array set to 0,
    /// providing a clean slate for managing L2 page table entries.
    ///
    /// # Examples
    ///
    /// ```
    /// const PAGE_TABLE_SIZE: usize = 256;
    /// const TABLE: L2PageTable = L2PageTable::new();
    /// assert_eq!(TABLE.0, [0; PAGE_TABLE_SIZE]);
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
    /// Attempts to create a new L2 small page table entry using an available free page slot.
    /// 
    /// The function searches through the page table entries (from 0 to `PAGE_TABLE_SIZE`) to locate the first unused page.
    /// When a free page is found, it marks the page as used and computes its physical address using `BASE_ADDRESS` and the page's index shifted by `PAGE_SIZE_BITS`.
    /// The new entry is initialized with the provided optional ASID, a default virtual address of 0, and full access permissions.
    /// It returns `None` if no unused page slot is available.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use your_module::L2SmallPageTableEntry; // Adjust the import path as needed.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(42)) {
    ///     // Successfully allocated a new L2 page table entry.
    ///     println!("New entry allocated with physical address: {}", entry.physical_address);
    /// } else {
    ///     // Handle the failure when no free page is available.
    ///     eprintln!("No free page available to allocate a new entry.");
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

    /// Creates an empty L2SmallPageTableEntry with default values.
    ///
    /// The returned entry has no associated ASID (set to `None`), and both its virtual and physical addresses are initialized to zero. The access permissions are set to full access.
    ///
    /// # Examples
    ///
    /// ```
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

    /// Sets the Address Space Identifier (ASID) in the system control register for the page table entry.
    ///
    /// If the entry's `asid` field contains a value, this method uses inline assembly to write the value
    /// to the coprocessor register via the `mcr` instruction. If `asid` is `None`, the method returns without
    /// performing any action.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l2::L2SmallPageTableEntry;
    ///
    /// // Create a new entry with an ASID. In practice, use the appropriate constructor.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(1)) {
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

    /// Invalidates the Translation Lookaside Buffer (TLB) entry for the page associated with this entry.
    ///
    /// The method calculates the target address by aligning the virtual address down to the page boundary and combining it with the entry’s address space identifier (ASID), if available. It then uses an inline assembly instruction to trigger the TLB invalidation for that computed address.
    ///
    /// # Safety
    /// This function uses unsafe inline assembly to interact directly with processor registers. Ensure that the virtual address and ASID are correct for the intended TLB entry.
    ///
    /// # Examples
    ///
    /// ```
    /// // Dummy struct for demonstration purposes.
    /// struct L2SmallPageTableEntry {
    ///     asid: Option<u32>,
    ///     virtual_address: u32,
    /// }
    ///
    /// impl L2SmallPageTableEntry {
    ///     fn invalidate_tlb(&self) {
    ///         unsafe {
    ///             std::arch::asm!("mcr p15, 0, {mva}, c8, c7, 1", mva = in(reg) (self.virtual_address & !0xFFF) | self.asid.unwrap_or(0));
    ///         }
    ///     }
    /// }
    ///
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(1),
    ///     virtual_address: 0x8000_1000,
    /// };
    /// entry.invalidate_tlb();
    /// ```
    fn invalidate_tlb(&self) {
        unsafe {
            asm!("mcr p15, 0, {mva}, c8, c7, 1", mva = in(reg) (self.virtual_address & !0xFFF) | self.asid.unwrap_or(0));
        };
    }

    /// Registers the L2 page table entry.
    ///
    /// This method sets the entry's address space identifier (ASID) and writes the entry's
    /// converted representation into the global L2 page table at the index determined by shifting
    /// its virtual address by the number of page size bits.
    ///
    /// # Safety
    ///
    /// This operation performs an unsafe memory write to the global L2 page table.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume a valid L2SmallPageTableEntry is available
    /// let entry = L2SmallPageTableEntry::try_new(Some(1)).unwrap();
    /// entry.register();
    /// ```
    pub fn register(&self) {
        self.set_asid();

        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] = self.into();
        }
    }

    /// Unregisters this page table entry from the Level 2 page table.
    /// 
    /// This method resets the corresponding entry in the Level 2 page table to a fault state,
    /// marks the associated physical page as available, and invalidates the TLB entry for the
    /// entry's virtual address.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Attempt to create a new page table entry with an optional ASID.
    /// if let Some(entry) = L2SmallPageTableEntry::try_new(Some(1)) {
    ///     // Register the entry to set up its mapping.
    ///     entry.register();
    ///     
    ///     // When the entry is no longer needed, unregister it to clear its mapping and free the page.
    ///     entry.unregister();
    /// }
    /// ```
    pub fn unregister(&self) {
        unsafe {
            LEVEL2_PAGE_TABLE.0[self.virtual_address as usize >> PAGE_SIZE_BITS] =
                L2_FAULT_PAGE_TABLE_ENTRY;
            USED_PAGES[(self.physical_address - BASE_ADDRESS) as usize >> PAGE_SIZE_BITS] = false;
        }

        self.invalidate_tlb();
    }

    /// Returns the starting virtual address for this page table entry.
    ///
    /// This method returns the virtual address that marks the beginning of the memory region associated with the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l2::{L2SmallPageTableEntry, AccessPermissions};
    ///
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(1),
    ///     virtual_address: 0x1000,
    ///     physical_address: 0x2000,
    ///     access_permissions: AccessPermissions::FullAccess,
    /// };
    /// assert_eq!(entry.start(), 0x1000);
    /// ```
    pub fn start(&self) -> u32 {
        self.virtual_address
    }

    /// Returns the ending virtual address for the page table entry.
    ///
    /// The ending address is determined by adding the system-defined page size to the entry's
    /// base virtual address and subtracting 4 bytes. This calculation yields the last valid
    /// address within the allocated page.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming a page size of 4096 bytes and an entry with a base virtual address of 0x1000:
    /// // The end address will be 0x1000 + 4096 - 4.
    /// let entry = L2SmallPageTableEntry {
    ///     virtual_address: 0x1000,
    ///     // other fields initialized to valid dummy values as needed
    ///     ..Default::default()
    /// };
    /// assert_eq!(entry.end(), 0x1000 + 4096 - 4);
    /// ```
    pub fn end(&self) -> u32 {
        self.virtual_address + PAGE_SIZE - 4
    }
}

impl From<&L2SmallPageTableEntry> for u32 {
    /// Converts an L2 small page table entry into its 32-bit encoded representation.
    ///
    /// This conversion packs the physical address, a flag indicating if an address space identifier (ASID)
    /// is present, and the access permissions into a single 32-bit value suitable for use in the MMU.
    ///
    /// # Examples
    ///
    /// ```
    /// let entry = L2SmallPageTableEntry {
    ///     asid: Some(1),
    ///     virtual_address: 0x2000,
    ///     physical_address: 0x1000,
    ///     permissions: AccessPermissions::Full,
    /// };
    ///
    /// let encoded: u32 = u32::from(&entry);
    ///
    /// // The resulting encoded value includes a constant flag (0b10) among other fields.
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
    /// Converts an `AccessPermissions` variant into its corresponding 32-bit unsigned integer representation.
    /// 
    /// This conversion maps the `Full` variant to `(0b11 << 4)`, encoding full access permissions.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let permission = AccessPermissions::Full;
    /// let encoded = u32::from(&permission);
    /// assert_eq!(encoded, 0b11 << 4);
    /// ```
    fn from(value: &AccessPermissions) -> Self {
        match value {
            AccessPermissions::Full => 0b11 << 4,
        }
    }
}
