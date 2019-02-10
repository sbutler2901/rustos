use core::alloc::{Alloc, GlobalAlloc, Layout};
use core::ptr::NonNull;
use self::bitmap_allocator::*;
use x86_64::structures::paging::Page;
use spin::Mutex;

mod bitmap_allocator;

pub struct HeapAllocator {
    allocator: Mutex<BitmapAllocator>,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        HeapAllocator {
            allocator: Mutex::new(BitmapAllocator::new())
        }
    }

    pub fn init(&self, heap_page: Page) {
        self.allocator.lock().init(heap_page.start_address().as_u64(), heap_page.size());
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout).ok().map_or(0 as *mut u8, |addr| addr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.lock().dealloc(NonNull::new_unchecked(ptr), layout);
    }
}
