use x86_64::PhysAddr;
use x86_64:: structures::paging::PageTable;

/// Returns the physical address for the given virtual address, or `None` if the
/// virtual address is not mapped.
pub fn translate_addr(addr: usize) -> Option<PhysAddr> {
    // introduce variables for the recursive index and the sign extension bits
    // TODO: Don't hardcode these values
    let r = 0o777; // recursive index = 511
    let sign = 0o177777 << 48; // sign extension: last 16 bits of 64 (64-48) set to 1

    // retrieve the page table indices of the address that we want to translate
    // each index represented with 9 bits: 512 indices
    let l4_idx = (addr >> 39) & 0o777; // level 4 index
    let l3_idx = (addr >> 30) & 0o777; // level 3 index
    let l2_idx = (addr >> 21) & 0o777; // level 2 index
    let l1_idx = (addr >> 12) & 0o777; // level 1 index
    let page_offset = addr & 0o7777;

    // calculate the table addresses
    let level_4_table_addr =
        sign | (r << 39) | (r << 30) | (r << 21) | (r << 12);
    let level_3_table_addr =
        sign | (r << 39) | (r << 30) | (r << 21) | (l4_idx << 12);
    let level_2_table_addr =
        sign | (r << 39) | (r << 30) | (l4_idx << 21) | (l3_idx << 12);
    let level_1_table_addr =
        sign | (r << 39) | (l4_idx << 30) | (l3_idx << 21) | (l2_idx << 12);

    // check that level 4 entry is mapped
    let level_4_table = unsafe { &*(level_4_table_addr as *const PageTable) };
    if level_4_table[l4_idx].addr().is_null() {
        return None;
    }

    // check that level 3 entry is mapped
    let level_3_table = unsafe { &*(level_3_table_addr as *const PageTable) };
    if level_3_table[l3_idx].addr().is_null() {
        return None;
    }

    // check that level 2 entry is mapped
    let level_2_table = unsafe { &*(level_2_table_addr as *const PageTable) };
    if level_2_table[l2_idx].addr().is_null() {
        return None;
    }

    // check that level 1 entry is mapped and retrieve physical address from it
    let level_1_table = unsafe { &*(level_1_table_addr as *const PageTable) };
    let phys_addr = level_1_table[l1_idx].addr();
    if phys_addr.is_null() {
        return None;
    }

    Some(phys_addr + page_offset)
}