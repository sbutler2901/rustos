#![no_std]  // dont linke Rust stdlib
// only enable no_main macro in non-test mode
// which prevents a main from being injected since this is a kernel
#![cfg_attr(not(test), no_main)]
// silence certain warnings when testing is being performed
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(alloc)]

#[macro_use]
extern crate rust_os;
extern crate x86_64;
extern crate bootloader;
extern crate alloc;

use core::panic::PanicInfo;
use rust_os::{gdt, interrupts, memory};
use bootloader::{bootinfo::BootInfo, entry_point};
use alloc::vec::Vec;
use core::alloc::GlobalAlloc;

entry_point!(kernel_main);

// The function expected in linker for the start of the program
#[cfg(not(test))] // only compile when test flag is not set
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    gdt::init();    // load GDT
    interrupts::init_idt();     // load IDT

    // Initialize PICs for hardware interrupts
    // unsafe: possible undefined behavior if PIC misconfigured
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();     // enables external interrupts

    memory::init(boot_info);

    for region in boot_info.memory_map.iter() {
        serial_println!("{:?} {:?}", region.region_type, region.range);
    }

    let layout = core::alloc::Layout::new::<u32>();

    let test0 = unsafe { rust_os::HEAP_ALLOCATOR.alloc(layout) };
    serial_println!("main test0: {:?}", test0);

    let test1 = unsafe { rust_os::HEAP_ALLOCATOR.alloc(layout) };
    serial_println!("main test1: {:?}", test1);

    unsafe { rust_os::HEAP_ALLOCATOR.dealloc(test0, layout)}
    unsafe { rust_os::HEAP_ALLOCATOR.dealloc(test1, layout)}

    let mut test: Vec<u64> = Vec::new();

    for item in 0..1 {
        test.push(item);
    }


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
