use core::ops::Range;

use super::l2::L2PageTable;

const PAGE_SIZE: u32 = 1 << 20;
const PAGE_SIZE_BITS: u32 = 20;
const PAGE_TABLE_SIZE: usize = 4096;

/// Initializes memory regions for kernel and peripheral devices by enabling corresponding page table entries with full access permissions.
///
/// This function defines two memory ranges—one for kernel memory and one for peripheral memory—and configures each range in the page table with full access permissions. It is a key step in setting up the memory management system.
///
/// # Examples
///
/// ```
/// // Initialize the memory management system
/// initialize();
/// ```
pub fn initialize() {
    let peripheral_memory: Range<u32> = 0x4400_0000..0x8000_0000;
    let kernel_memory: Range<u32> = 0x4020_0000..0x4040_0000;

    enable_memory_range(kernel_memory, AccessPermissions::Full);
    enable_memory_range(peripheral_memory, AccessPermissions::Full);
}

/// Enables all pages in the specified memory range with the desired access permissions.
///
/// This function divides the given memory range into pages of size `PAGE_SIZE` and creates an
/// `L1SectionPageTableEntry` for each page using the provided `permissions`. Each generated entry
/// is then stored at the corresponding index in the global level 1 page table.
///
/// # Examples
///
/// ```
/// # use std::ops::Range;
/// # const PAGE_SIZE: u32 = 1 << 20;
/// # const PAGE_SIZE_BITS: u32 = 20;
/// # #[derive(Clone, Copy)]
/// # enum AccessPermissions { Full, Privileged, UserReadOnly }
/// # struct L1SectionPageTableEntry;
/// # impl L1SectionPageTableEntry {
/// #     fn new(address: u32, permissions: AccessPermissions) -> Self { L1SectionPageTableEntry }
/// # }
/// # struct L1PageTable([u32; 4096]);
/// # static mut LEVEL1_PAGE_TABLE: L1PageTable = L1PageTable([0; 4096]);
/// # impl From<L1SectionPageTableEntry> for u32 {
/// #     fn from(_: L1SectionPageTableEntry) -> Self { 0 }
/// # }
///
/// let memory_range: Range<u32> = 0..PAGE_SIZE;
/// enable_memory_range(memory_range, AccessPermissions::Full);
/// // Each page in the specified memory range is now enabled with full access permissions.
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
    /// Creates a new L1PageTable with all entries initialized to zero.
    ///
    /// This constant constructor returns an instance of L1PageTable, with its internal array set entirely to zero,
    /// ensuring a clean state for the page table.
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
    /// Constructs a new L1 section page table entry with the specified address and access permissions.
    ///
    /// This function creates a new entry for the level 1 page table. The provided address serves as the base address for the memory region and the access permissions determine the allowed operations on that region.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::mmu::{L1SectionPageTableEntry, AccessPermissions};
    ///
    /// let entry = L1SectionPageTableEntry::new(0x8000_0000, AccessPermissions::Full);
    /// assert_eq!(entry.address, 0x8000_0000);
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
    /// Converts an `L1SectionPageTableEntry` into its 32-bit hardware representation.
    ///
    /// This conversion combines the page's base address with the corresponding access permissions
    /// and a flag (0b10) to produce a value suitable for insertion into a level 1 page table.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_module::{L1SectionPageTableEntry, AccessPermissions};
    ///
    /// let entry = L1SectionPageTableEntry::new(0x0010_0000, AccessPermissions::Full);
    /// let converted: u32 = entry.into();
    ///
    /// // The converted value contains the original address, encoded permissions, and the flag 0b10.
    /// assert_eq!(converted & 0b10, 0b10);
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
    /// Converts a `L1PointerTableEntry` into its 32-bit representation with an applied flag.
    ///
    /// This function casts the contained pointer to a 32-bit unsigned integer and sets the least
    /// significant bit (0b01), marking the value as a pointer table entry.
    ///
    /// # Examples
    ///
    /// ```
    /// // For demonstration purposes, a dummy pointer is used. In real applications, ensure the pointer is valid.
    /// let dummy_ptr = 0x2000 as *const ();
    /// let entry = L1PointerTableEntry { table: dummy_ptr };
    /// let value: u32 = entry.into();
    /// assert_eq!(value, dummy_ptr as u32 | 0b01);
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
    /// Converts an `AccessPermissions` variant into its corresponding 32-bit bit pattern.
    ///
    /// The conversion shifts the designated permission bits 10 positions to the left:
    /// - `AccessPermissions::UserReadOnly` converts to `0b10 << 10`
    /// - `AccessPermissions::Privileged` converts to `0b01 << 10`
    /// - `AccessPermissions::Full` converts to `0b11 << 10`
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::mmu::l1::AccessPermissions;
    ///
    /// let bits: u32 = AccessPermissions::Full.into();
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
