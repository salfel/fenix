use core::ops::Range;

use super::l2::L2PageTable;

const PAGE_SIZE: u32 = 1 << 20;
const PAGE_SIZE_BITS: u32 = 20;
const PAGE_TABLE_SIZE: usize = 4096;

/// Initializes memory regions for kernel and peripheral memory with full access permissions.
///
/// This function sets up the system's memory management by defining the ranges for kernel memory 
/// (0x4020_0000 to 0x4040_0000) and peripheral memory (0x4400_0000 to 0x8000_0000). It then
/// enables these regions by applying full access permissions to each page in the specified ranges.
///
/// # Examples
///
/// ```
/// // Initialize memory access permissions for kernel and peripheral regions.
/// initialize();
/// ```
pub fn initialize() {
    let peripheral_memory: Range<u32> = 0x4400_0000..0x8000_0000;
    let kernel_memory: Range<u32> = 0x4020_0000..0x4040_0000;

    enable_memory_range(kernel_memory, AccessPermissions::Full);
    enable_memory_range(peripheral_memory, AccessPermissions::Full);
}

/// Enables a specified memory range by updating the level 1 page table with the provided access permissions.
///
/// This function iterates through the given range in increments of `PAGE_SIZE`. For each page address, it creates
/// an `L1SectionPageTableEntry` with the specified `permissions` and writes its 32-bit representation into the
/// global `LEVEL1_PAGE_TABLE`. The appropriate table index is computed by shifting the page address right by
/// `PAGE_SIZE_BITS`.
///
/// # Examples
///
/// ```
/// use core::ops::Range;
///
/// // Define a memory range starting at 0x1000_0000, spanning 5 pages.
/// let range: Range<u32> = 0x1000_0000..(0x1000_0000 + 5 * PAGE_SIZE);
/// enable_memory_range(range, AccessPermissions::Full);
/// ```
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
    /// Creates a new level 1 page table with all entries initialized to zero.
    ///
    /// This `const` function returns an instance of `L1PageTable` containing an array of `u32`
    /// values, with the size defined by `PAGE_TABLE_SIZE`. All entries in the table are set to zero,
    /// ensuring a clean slate for memory management.
    ///
    /// # Examples
    ///
    /// ```
    /// let page_table = L1PageTable::new();
    /// assert!(page_table.0.iter().all(|&entry| entry == 0));
    /// ```
    const fn new() -> Self {
        L1PageTable([0; PAGE_TABLE_SIZE])
    }
}

pub struct L1SectionPageTableEntry {
    address: u32,
    access_permissions: AccessPermissions,
}

impl L1SectionPageTableEntry {
    /// Creates a new `L1SectionPageTableEntry` with the specified page address and access permissions.
    ///
    /// This function initializes a section page table entry that holds the starting address of a memory page
    /// along with its associated access permissions, preparing it for conversion into a 32-bit representation
    /// for memory management.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_module::AccessPermissions;
    /// use your_module::L1SectionPageTableEntry;
    ///
    /// let entry = L1SectionPageTableEntry::new(0x80000000, AccessPermissions::Full);
    /// assert_eq!(entry.address, 0x80000000);
    /// ```
    fn new(address: u32, access_permissions: AccessPermissions) -> Self {
        L1SectionPageTableEntry {
            address,
            access_permissions,
        }
    }
}

impl From<L1SectionPageTableEntry> for u32 {
    /// Converts a Level 1 section page table entry into its 32-bit integer representation.
    ///
    /// This conversion merges the entry's memory address with its access permission bits and sets a section flag (0b10).
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a section page table entry with a sample address and full access permissions.
    /// let entry = L1SectionPageTableEntry::new(0x1000, AccessPermissions::Full);
    /// // Convert the entry into its 32-bit representation.
    /// let value: u32 = entry.into();
    /// // Verify that the section flag (0b10) is present in the resulting value.
    /// assert_eq!(value & 0b10, 0b10);
    /// ```
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
    /// Converts an L1PointerTableEntry into its 32-bit integer representation.
    ///
    /// This conversion casts the contained table pointer to a `u32` and sets a flag in the least
    /// significant bit by performing a bitwise OR with `0b01`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assume an L1PointerTableEntry with a table pointer set to a valid address.
    /// let entry = L1PointerTableEntry { table: 0x1000 };
    /// let value: u32 = entry.into();
    /// assert_eq!(value, (0x1000 as u32) | 0b01);
    /// ```
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
    /// Converts an `AccessPermissions` variant into its corresponding 32-bit flag representation.
    ///
    /// Each permission level is mapped to a specific bit pattern via a left shift by 10:
    /// - `AccessPermissions::UserReadOnly` is converted to `0b10 << 10`.
    /// - `AccessPermissions::Privileged` is converted to `0b01 << 10`.
    /// - `AccessPermissions::Full` is converted to `0b11 << 10`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l1::AccessPermissions;
    ///
    /// let flag: u32 = AccessPermissions::UserReadOnly.into();
    /// assert_eq!(flag, 0b10 << 10);
    /// ```
    fn from(value: AccessPermissions) -> Self {
        match value {
            AccessPermissions::UserReadOnly => 0b10 << 10,
            AccessPermissions::Privileged => 0b01 << 10,
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}
