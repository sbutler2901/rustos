#![no_std]  // dont linke Rust stdlib
// only enable no_main macro in non-test mode
// which prevents a main from being injected since this is a kernel
#![cfg_attr(not(test), no_main)]
// silence certain warnings when testing is being performed
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(asm)]

#[macro_use]
extern crate rust_os;
extern crate x86_64;
extern crate bootloader;

use core::panic::PanicInfo;
use bootloader::{bootinfo::BootInfo, entry_point};
//use x86_64::structures::paging::RecursivePageTable;

entry_point!(kernel_main);

// The function expected in linker for the start of the program
#[cfg(not(test))] // only compile when test flag is not set
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = memory::EmptyFrameAllocator;

    // map an unused page
    // Page 0 is usual always left empty so that de-referencing a null pointer
    // will always cause a page fault. This page is also already has all necessary page tables
    // pre-allocated, so the frame_allocator and be a dummy implementation
    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    println!("It did not crash!");
    rust_os::hlt_loop();
}

// Defines the method to use in case of a panic
#[cfg(not(test))]       // only compile when test flag is not set
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}
