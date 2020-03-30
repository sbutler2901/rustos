use x86_64::{
    structures::paging::{
        Page, PhysFrame, Mapper, Size4KiB, FrameAllocator, UnusedPhysFrame,
        OffsetPageTable, PageTable
    },
    VirtAddr,
    PhysAddr
};

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        None
    }
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    // get mutable reference to the level 4 page table
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // read the physical frame of the active level 4 table from the CR3 register.
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    // get the virtual address of the table using the physical address
    // and the physical memory offset
    let virt = physical_memory_offset + phys.as_u64();

    // create a raw pointer to the table
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // create and return a mutable reference from the raw pointer
    &mut *page_table_ptr // unsafe
}

/// Creates an example mapping for the given page to frame `0xb8000`.
/// This is the vga frame, so data written will be displayed
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // FIXME: ONLY FOR TEMPORARY TESTING
    // unsafe because frame is already mapped for vga buffer
    let unused_frame = unsafe { UnusedPhysFrame::new(frame) };
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = mapper.map_to(page, unused_frame, flags, frame_allocator);
    map_to_result.expect("map_to failed").flush();
}