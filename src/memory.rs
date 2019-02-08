// in src/memory.rs

//use x86_64::structures::paging::{Mapper, Page, PageTable, RecursivePageTable};
use x86_64::{VirtAddr, PhysAddr, structures::paging::*};
//use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

// Since we know that no new page tables are required for the address
// 0x1000, a frame allocator that always returns None suffices.
impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

pub struct BootInfoFrameAllocator<I> where I: Iterator<Item = PhysFrame> {
    // can be initialized with an arbitrary Iterator of frames.
    // This allows us to just delegate alloc calls to the Iterator::next method.
    frames: I,
}

impl<I> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<I>
    where I: Iterator<Item = PhysFrame>
{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.frames.next()
    }
}

/// Create a FrameAllocator from the passed memory map
pub fn init_frame_allocator(
    memory_map: &'static MemoryMap,
) -> BootInfoFrameAllocator<impl Iterator<Item = PhysFrame>> {
    // get usable regions from memory map
    let regions = memory_map
        // convert the memory map to an iterator of MemoryRegions
        .iter()
        // skip any reserved or otherwise unavailable regions.
        // rames that are used by our kernel (code, data or stack) or to store the boot
        // information are already marked as InUse or similar by bootloader
        .filter(|r| r.region_type == MemoryRegionType::Usable);

    // transform our iterator of memory regions to an iterator of address ranges.
    let addr_ranges = regions.map(|r| r.range.start_addr()..r.range.end_addr());

    //We convert each range to an iterator through the into_iter method and
    // then choose every 4096th address using step_by. Since 4096 bytes (= 4 KiB) is the page size,
    // we get the start address of each frame. The bootloader page aligns all usable memory areas
    // so that we don't need any alignment or rounding code here. By using flat_map instead of map,
    // we get an Iterator<Item = u64> instead of an Iterator<Item = Iterator<Item = u64>>.
    let frame_addresses = addr_ranges.flat_map(|r| r.into_iter().step_by(4096));

    // create `PhysFrame` types from the start addresses.
    // we convert the start addresses to PhysFrame types to construct the desired Iterator<Item = PhysFrame>
    let frames = frame_addresses.map(|addr| {
        PhysFrame::containing_address(PhysAddr::new(addr))
    });

    BootInfoFrameAllocator { frames }
}

// creates a sample mapping.
// Mutiple reference to Recursive page table because needs to modify entries
pub fn create_example_mapping(
    recursive_page_table: &mut RecursivePageTable,
    // The Size4KiB argument in the trait implementation is needed because the Page and PhysFrame
    // types are generic over the PageSize trait to work with both standard 4KiB pages and huge 2MiB/1GiB pages.
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    // Bootloader occupies first MB of virtual memory, so know level 1 page table
    // is valid for this reasons. So can use page at 0x1000.
//    let page: Page = Page::containing_address(VirtAddr::new(0x1000));

    // Uses frame allocator to allocate new frame to store page tables and new page
    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    // The target frame will be 0xb8000, the VGA buffer's frame for easy mapping testing.
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    // Present flag required for all valid page table entries
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // map_to from Mapper trait to map page at address 0x1000 to physical frame
        // at 0xb8000. Unsafe because possible to break memory safety with invalid arguments.
        // frame_allocator must implement FrameAllocator trait.The map_to method needs this argument
        // because it might need unused frames for creating new page tables.
        recursive_page_table.map_to(page, frame, flags, frame_allocator)
    };
    // Sample code so use expect to panic in case of error.
    // Return MapperFlush type provides easy way to flush newly mapped page from TLB
    map_to_result.expect("map_to failed").flush();
}

/// Creates a RecursivePageTable instance from the level 4 address.
///
/// This function is unsafe because it can break memory safety if an invalid
/// address is passed.
pub unsafe fn init(level_4_table_addr: usize) -> RecursivePageTable<'static> {
    /// Rust currently treats the whole body of unsafe functions as an unsafe
    /// block, which makes it difficult to see which operations are unsafe. To
    /// limit the scope of unsafe we use a safe inner function.
    /// RFC: https://github.com/rust-lang/rfcs/pull/2585
    fn init_inner(level_4_table_addr: usize) -> RecursivePageTable<'static> {
        let level_4_table_ptr = level_4_table_addr as *mut PageTable;
        let level_4_table = unsafe { &mut *level_4_table_ptr };
        RecursivePageTable::new(level_4_table).unwrap()
    }

    init_inner(level_4_table_addr)
}

/// Returns the physical address for the given virtual address, or `None` if
/// the virtual address is not mapped.
pub fn translate_addr(addr: u64, recursive_page_table: &RecursivePageTable)
                      -> Option<PhysAddr>
{
    let addr = VirtAddr::new(addr);
    let page: Page = Page::containing_address(addr);

    // perform the translation
    let frame = recursive_page_table.translate_page(page);
    frame.map(|frame| frame.start_address() + u64::from(addr.page_offset()))
}