use core::alloc::{Alloc, GlobalAlloc, Layout};
use core::ptr::NonNull;
//use self::bitmap_allocator::*;
use self::linked_list_allocator::*;
use spin::Mutex;

//mod bitmap_allocator;
mod linked_list_allocator;

pub const HEAP_START: u64 = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024;
pub const MIN_ALLOC_SIZE: usize = 16;

pub fn min_size() -> usize {
    MIN_ALLOC_SIZE
}

pub struct HeapAllocator {
    allocator: Mutex<LinkedListAllocator>,
}

impl HeapAllocator {
    pub const fn new() -> Self {
        HeapAllocator {
            allocator: Mutex::new(LinkedListAllocator::new())
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.allocator.lock().init(heap_start, heap_size as usize);
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

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
