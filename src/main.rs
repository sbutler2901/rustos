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

use rust_os::{println, serial_println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

// The function expected in linker for the start of the program
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    rust_os::init();

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
