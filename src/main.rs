#![no_std]  // dont linke Rust stdlib
// only enable no_main macro in non-test mode
// which prevents a main from being injected since this is a kernel
// #![cfg_attr(not(test), no_main)]
// silence certain warnings when testing is being performed
// #![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rust_os::{println, serial_println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

// The function expected in linker for the start of the program
#[cfg(not(test))] // only compile when test flag is not set
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    rust_os::init();

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
