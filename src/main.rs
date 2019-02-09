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
use rust_os::{gdt, interrupts};
use rust_os::memory::{init, translate_addr, create_example_mapping, init_frame_allocator};
use bootloader::{bootinfo::BootInfo, entry_point};
use x86_64::structures::paging::RecursivePageTable;

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

    let mut recursive_page_table: RecursivePageTable = unsafe { init(boot_info.p4_table_addr as usize) };
    let mut frame_allocator = init_frame_allocator(&boot_info.memory_map);

    for region in boot_info.memory_map.iter() {
        serial_println!("{:?} {:?}", region.region_type, region.range);
    }

    // create mapping at 0x1000
    create_example_mapping(&mut recursive_page_table, &mut frame_allocator);

    // Write string New! to VGA buffer. Offsets by 900 since vga buffer pushes
    // top line off screen on next println
    // Only works because know level 1 page table is already mapped and don't need frame allocator
//    unsafe { (0x1900 as *mut u64).write_volatile(0xf021f077f065f04e)};

    // Writes to vga buffer with page mapped by frame allocator
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e)};

    // This address is identity mapped for VGA and so the translation doesn't change the address
    println!("0xb8000 -> {:?}", translate_addr(0xb8000, &recursive_page_table));

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
