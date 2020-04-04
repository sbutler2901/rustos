#![no_std]  // dont linke Rust stdlib
// silence certain warnings when testing is being performed
// #![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![no_main]
// replacing the default test framework (since stdlib is not available)
#![feature(custom_test_frameworks)]
// use the test_runner function created in the rust_os lib
#![test_runner(rust_os::test_runner)]
// Change the custom test framework's generated main name
// to prevent no_main from causing it to be ignored
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use rust_os::{println, serial_println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

entry_point!(kernel_main);

// The function expected in linker for the start of the program
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_os::allocator; // new import
    use rust_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::{VirtAddr};

    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    rust_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // the Box and Vec values printed will be their heap pointers
    // the address will begin with 0x4444_4444
    // because of the heap's defined starting virtual address

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));


    // Call the test framework entry point when testing
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    rust_os::hlt_loop();
}

// Defines the method to use in case of a panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

// Defines the method to use in case of a panic during testing
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
