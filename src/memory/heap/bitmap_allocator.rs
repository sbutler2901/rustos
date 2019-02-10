use core::alloc::{Alloc, Layout, AllocErr};
use core::ptr::NonNull;
use x86_64::VirtAddr;

const PAGE_SIZE: usize = 4096;
const BLOCK_SIZE: usize = 16;
const BLOCK_CNT: usize = PAGE_SIZE / BLOCK_SIZE;

// TODO: expand to support multiple blocks
struct BitmapBlock {
    start_addr: u64,
    size: usize,
    used: usize,
    block_size: usize,
    bitmap: [bool; BLOCK_CNT],
}

pub struct BitmapAllocator {
    block: BitmapBlock,
}

impl BitmapAllocator {
    pub const fn new() -> Self {
        BitmapAllocator {
            block: BitmapBlock {
                start_addr: 0,
                size: 0,
                used: 0,
                block_size: BLOCK_SIZE,
                bitmap: [false; BLOCK_CNT]
            }
        }
    }

    pub fn init(&mut self, start_addr: u64, size: u64) {
        self.block.start_addr = start_addr;
        self.block.size = size as usize;
    }
}

unsafe impl Alloc for BitmapAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        if (self.block.size - self.block.used) < layout.size() {
            return Result::Err(AllocErr);
        }

        let mut start_index: usize = 0;
        let mut end_index: usize = 0;
        let mut free_space: usize = 0;
        for (index, block) in self.block.bitmap.iter().enumerate() {
            if *block == false {
                free_space += self.block.block_size;
                if free_space >= layout.size() {
                    end_index = index;
                    break;
                }
            } else {
                start_index = index + 1;
                free_space = 0;
            }
        }
        if free_space < layout.size() {
            return Result::Err(AllocErr);
        }

        for mut index in start_index..=end_index {
            self.block.bitmap[index] = true;
        }

        let addr = VirtAddr::new(self.block.start_addr + (start_index * self.block.block_size) as u64);

        Result::Ok(NonNull::new_unchecked(addr.as_mut_ptr()))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let ptr_start_addr = VirtAddr::from_ptr(ptr.as_ptr()).as_u64() as usize;
        let start_index: usize = (ptr_start_addr - self.block.start_addr as usize) / self.block.block_size;
        let end_index: usize = (layout.size() / self.block.block_size) + start_index;

        for index in start_index..=end_index {
            self.block.bitmap[index] = false;
        }
    }
}

