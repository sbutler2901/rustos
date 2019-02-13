use core::alloc::{Alloc, Layout, AllocErr};
use core::ptr::NonNull;
use core::mem::{align_of, size_of};
use x86_64::VirtAddr;

struct Hole {
    size: usize,
    next: Option<&'static mut Hole>
}

impl Hole {
    pub fn get_next(&self) -> &Option<&'static mut Hole> {
        &self.next
    }
}

struct HoleList {
    first: Hole,
}

impl HoleList {
    pub const fn empty() -> HoleList {
        HoleList {
            first: Hole {
                size: 0,
                next: None
            }
        }
    }

    pub unsafe fn new(start_addr: usize, size: usize) -> HoleList {
        let hole_ptr = start_addr as *mut Hole;
        hole_ptr.write(Hole {
            size,
            next: None,
        });

        HoleList {
            first: Hole {
                size: 0,
                next: Some(&mut *hole_ptr),
            }
        }
    }

    pub fn min_size() -> usize {
        size_of::<usize>() * 2
    }

    pub fn best_fit_allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        assert!(layout.size() >= Self::min_size());

        let test = self.first.get_next();
        let hmm: &mut Hole = (*test).unwrap();
//        let mut prev_hole: Option<&Hole> = None;
//        let mut best_fit: Option<&mut Hole> = None;
//        let mut current: &Hole = &self.first;
//        loop {
//            if let Some(hole) = current.next.as_mut() {
//                if hole.size == layout.size() {
//                    best_fit = Some(hole);
//                    break;
//                } else if hole.size > layout.size() {
//                    if let Some(current_best) = best_fit {
//                        if hole.size < current_best.size {
//                            best_fit = Some(hole);
//                        }
//                    } else {
//                        best_fit = Some(hole);
//                    }
//                }
//                prev_hole = Some(current);
//                current = hole;
//            } else {
//                break;
//            }
//        }
//        if let Some(best) = best_fit {
//            if let Some(mut prev) = prev_hole {
//                prev.next = best.next;
//            }
//        } else {
//            return Err(AllocErr);
//        }

        unsafe {
            let tmp = VirtAddr::new(0x01u64).as_mut_ptr();
            Result::Ok(NonNull::new_unchecked(tmp))
        }
    }
}

pub struct LinkedListAllocator {
    start_addr: usize,
    size: usize,
    holes: HoleList,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            start_addr: 0,
            size: 0,
            holes: HoleList::empty(),
        }
    }

    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        self.start_addr = start_addr;
        self.size = size;
        self.holes = HoleList::new(start_addr, size);
    }

    pub fn best_fit_allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let mut size = layout.size();
        serial_println!("size 0: {:?}", size);
        if size < HoleList::min_size() {
            size = HoleList::min_size();
        }
        serial_println!("size 1: {:?}", size);
        let size = super::align_up(size, align_of::<Hole>());
        serial_println!("size 2: {:?}", size);
        let layout = Layout::from_size_align(size, layout.align()).unwrap();
        serial_println!("layout: size: {:?}, align: {:?}", layout.size(), layout.align());

        self.holes.best_fit_allocate(layout)
    }

    pub fn deallocate(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        unimplemented!()
    }

}

unsafe impl Alloc for LinkedListAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        self.best_fit_allocate(layout)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.deallocate(ptr, layout)
    }
}
