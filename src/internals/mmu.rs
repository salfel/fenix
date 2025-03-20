#[link_section = "level_1_page_table"]
#[no_mangle]
static mut LEVEL1_PAGE_TABLE: Level1PageTable = Level1PageTable::new();

struct Level1PageTable([PageTableEntry; 4096]);

impl Level1PageTable {
    const fn new() -> Self {
        Self([const { PageTableEntry::new() }; 4096])
    }
}

struct PageTableEntry(u32);

impl PageTableEntry {
    const fn new() -> Self {
        Self(0)
    }
}
