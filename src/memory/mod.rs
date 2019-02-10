pub mod paging;
pub mod heap;

use x86_64::{VirtAddr, structures::paging::*};
use bootloader::bootinfo::BootInfo;
//use x86_64::structures::paging::PageTableFlags as Flags;

pub fn init(boot_info: &'static BootInfo) {
    let mut recursive_page_table: RecursivePageTable = unsafe { self::paging::init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = self::paging::init_frame_allocator(&boot_info.memory_map);

    let page: Page = Page::containing_address(VirtAddr::new(0x1000));   // TODO: determine best place for heap
    let heap_frame = frame_allocator.allocate_frame().unwrap();
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = unsafe {
        recursive_page_table.map_to(page, heap_frame, flags, &mut frame_allocator)
    };
    map_to_result.expect("map_to failed for heap allocations").flush();

    ::HEAP_ALLOCATOR.init(page);

//    paging::create_example_mapping(&mut recursive_page_table, &mut frame_allocator);

}
