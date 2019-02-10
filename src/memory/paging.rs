use x86_64::{VirtAddr, PhysAddr, structures::paging::*};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// A FrameAllocator that returns the next available free frame
pub struct BootInfoFrameAllocator<I>
    where I: Iterator<Item = PhysFrame>
{
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

// Creates a RecursivePageTable instance from the level 4 address.
// Unsafe because it can break memory safety if an invalid address is passed.
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

/// Create a FrameAllocator from the passed memory map
pub fn init_frame_allocator(
    memory_map: &'static MemoryMap,
) -> BootInfoFrameAllocator<impl Iterator<Item = PhysFrame>> {
    // get usable regions from memory map
    let regions = memory_map
        // convert the memory map to an iterator of MemoryRegions
        .iter()
        // skip any reserved or otherwise unavailable regions.
        // Regions that are used by our kernel (code, data or stack) or to store the boot
        // information are already marked as InUse or similar by bootloader
        .filter(|r| r.region_type == MemoryRegionType::Usable);

    // transform our iterator of memory regions to an iterator of address ranges.
    let addr_ranges = regions.map(|r| r.range.start_addr()..r.range.end_addr());

    // We convert each range to an iterator through the into_iter method and
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
