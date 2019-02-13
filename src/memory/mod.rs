pub mod paging;
pub mod heap;

use x86_64::{VirtAddr, structures::paging::*};
use bootloader::bootinfo::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    let mut recursive_page_table: RecursivePageTable = unsafe { self::paging::init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = self::paging::init_frame_allocator(&boot_info.memory_map);

    use self::heap::{HEAP_START, HEAP_SIZE};

    let heap_start_page: Page = Page::containing_address(VirtAddr::new(HEAP_START));   // TODO: determine best place for heap
    let heap_end_page: Page = Page::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE - 1 ));
    let heap_page_range = Page::range_inclusive(heap_start_page, heap_end_page);

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in heap_page_range {
        let heap_frame = frame_allocator.allocate_frame().unwrap();
        let map_to_result = unsafe {
            recursive_page_table.map_to(page, heap_frame, flags, &mut frame_allocator)
        };
        map_to_result.expect("map_to failed for heap allocations").flush();
    }

    unsafe {
        ::HEAP_ALLOCATOR.init(HEAP_START as usize, HEAP_SIZE as usize);
    }
}
