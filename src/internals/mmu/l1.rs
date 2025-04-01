use core::ops::Range;

use super::l2::L2PageTable;

const PAGE_SIZE: u32 = 1 << 20;
const PAGE_SIZE_BITS: u32 = 20;
const PAGE_TABLE_SIZE: usize = 4096;

/// Initializes memory regions for kernel and peripheral memory with full access permissions.
///
/// This function configures the memory management system by setting up two predefined
/// memory ranges:
/// - Kernel memory region: 0x4020_0000 to 0x4040_0000
/// - Peripheral memory region: 0x4400_0000 to 0x8000_0000
///
/// Each range is enabled with full access by invoking [`enable_memory_range`].
///
/// # Examples
///
/// ```
/// // Initialize memory management for kernel and peripherals.
/// initialize();
/// ```
pub fn initialize() {
    let peripheral_memory: Range<u32> = 0x4400_0000..0x8000_0000;
    let kernel_memory: Range<u32> = 0x4020_0000..0x4040_0000;

    enable_memory_range(kernel_memory, AccessPermissions::Full);
    enable_memory_range(peripheral_memory, AccessPermissions::Full);
}

/// Enables a specified memory range with the provided access permissions.
///
/// This function iterates over the memory addresses in the given `range` in increments of `PAGE_SIZE`. For each page,
/// it creates a new level 1 section page table entry using the provided `permissions` and writes it into the global
/// level 1 page table (`LEVEL1_PAGE_TABLE`). Ensure that the range is aligned to page boundaries since the function
/// relies on stepping by the page size and index calculation via shifting by `PAGE_SIZE_BITS`.
///
/// # Examples
///
/// ```
/// use your_crate::mmu::{enable_memory_range, AccessPermissions, PAGE_SIZE};
///
/// // Enable a memory range covering 5 pages with full access permissions.
/// let memory_range = 0..(PAGE_SIZE * 5);
/// enable_memory_range(memory_range, AccessPermissions::Full);
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
    /// Creates a new level one page table with all entries initialized to zero.
    ///
    /// This constant function returns a fresh instance of the page table. Every entry in the table is set to zero,
    /// ensuring that no memory position is pre-configured with any value.
    ///
    /// # Examples
    ///
    /// ```
    /// let page_table = L1PageTable::new();
    /// for entry in page_table.0.iter() {
    ///     assert_eq!(*entry, 0);
    /// }
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
    /// Creates a new level 1 section page table entry with a specific address and access permissions.
    ///
    /// This constructor initializes an L1SectionPageTableEntry by setting its address
    /// and associated access permissions. The address typically represents the starting
    /// address for the memory section this entry covers, while the access permissions
    /// define the allowed memory operations (e.g., Full, Privileged, or UserReadOnly).
    ///
    /// # Examples
    ///
    /// ```
    /// let entry = L1SectionPageTableEntry::new(0x00100000, AccessPermissions::Full);
    /// assert_eq!(entry.address, 0x00100000);
    /// assert_eq!(entry.access_permissions, AccessPermissions::Full);
    /// ```
    fn new(address: u32, access_permissions: AccessPermissions) -> Self {
        L1SectionPageTableEntry {
            address,
            access_permissions,
        }
    }
}

impl From<L1SectionPageTableEntry> for u32 {
    /// Converts an `L1SectionPageTableEntry` into a 32-bit page table entry.
    ///
    /// This function extracts the entry's page address and its associated access permissions,
    /// converts the permissions into a `u32` bitmask, and combines them with a fixed flag (`0b10`)
    /// using bitwise OR. The resulting `u32` value encodes all the necessary information for
    /// configuring the corresponding memory section in the level 1 page table.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::internals::mmu::{L1SectionPageTableEntry, AccessPermissions};
    ///
    /// // Create a section page table entry for a memory region starting at 0x0010_0000 with full access.
    /// let entry = L1SectionPageTableEntry::new(0x0010_0000, AccessPermissions::Full);
    /// let page_table_value: u32 = entry.into();
    ///
    /// // `page_table_value` now contains the bitwise OR of the address, the encoded access permissions,
    /// // and the fixed flag (`0b10`). The exact value depends on the internal encoding of AccessPermissions.
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
    /// Converts a `L1PointerTableEntry` into its raw `u32` representation.
    ///
    /// This function casts the pointer stored in the `L1PointerTableEntry` to a `u32`
    /// and sets the least significant bit (flag `0b01`) to indicate that the value
    /// refers to a level 2 page table.
    ///
    /// # Examples
    ///
    /// ```
    /// // For demonstration purposes, assume that L2PageTable is defined elsewhere.
    /// // Create a dummy pointer value as an example.
    /// let table_ptr = 0x1000 as *mut L2PageTable;
    /// let entry = L1PointerTableEntry { table: table_ptr };
    /// let raw: u32 = entry.into();
    /// assert_eq!(raw, table_ptr as u32 | 0b01);
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
    /// Converts an `AccessPermissions` variant into its corresponding encoded bit representation.
    ///
    /// The conversion shifts the base permission bits by 10 positions to obtain the proper value:
    /// - `AccessPermissions::UserReadOnly` is encoded as `0b10 << 10`
    /// - `AccessPermissions::Privileged` is encoded as `0b01 << 10`
    /// - `AccessPermissions::Full` is encoded as `0b11 << 10`
    ///
    /// This encoding is used to set access permissions in memory page table entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::internals::mmu::l1::AccessPermissions;
    ///
    /// let bits = u32::from(AccessPermissions::Full);
    /// assert_eq!(bits, 0b11 << 10);
    /// ```
    fn from(value: AccessPermissions) -> Self {
        match value {
            AccessPermissions::UserReadOnly => 0b10 << 10,
            AccessPermissions::Privileged => 0b01 << 10,
            AccessPermissions::Full => 0b11 << 10,
        }
    }
}
