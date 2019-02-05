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

use core::panic::PanicInfo;
use rust_os::{gdt, interrupts};

// The function expected in linker for the start of the program
#[cfg(not(test))] // only compile when test flag is not set
#[no_mangle]        // ensures function name is not mangled for usage by bootloader
#[allow(const_err)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    gdt::init();    // load GDT
    interrupts::init_idt();     // load IDT

    // Initialize PICs for hardware interrupts
    // unsafe: possible undefined behavior if PIC misconfigured
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();     // enables external interrupts

    // Testing zone

    use x86_64::structures::paging::PageTable;
    use x86_64::registers::control:: Cr3;

    // The control register 3 contains the currently active level 4 page table.
    // This give us the physical address of the page table
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at : {:?}", level_4_page_table.start_address());

    // Accessing physical memory directly not possible when paging is active. Need virtual page
    // that is mapped to the physical frame at address 0x1000.
    // The bootloader uses recursive page tables to map the last page of the virtual address space to the
    // physical frame of the level 4 page table: the subsequent memory address.
    let level_4_table_ptr = 0xffff_ffff_ffff_f000 as *const PageTable;  // cast as raw pointer to a PageTable
    // transform into a rust reference providing safe bounds checked indexing operations
    let level_4_table: &PageTable = unsafe { &* level_4_table_ptr };
    for i in 0..10 {
        println!("Entry {}: {:?}", i, level_4_table[i]);
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
