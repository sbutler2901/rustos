#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]  // enable usage of unstable x86-interrupt calling convention
#![feature(alloc_error_handler)]
// replacing the default test framework (since stdlib is not available)
#![feature(custom_test_frameworks)]
// use the test_runner function created in the rust_os lib
#![test_runner(crate::test_runner)]
// Change the custom test framework's generated main name
// to prevent no_main from causing it to be ignored
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

#[macro_use]
pub mod vga_buffer;
pub mod serial;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;
pub mod allocator;

pub fn init() {
    gdt::init(); // load GDT
    interrupts::init_idt(); // load IDT

    // Initialize PICs for hardware interrupts
    // unsafe: possible undefined behavior if PIC misconfigured
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // enables external interrupts
}

// Notify the CPU to halt until the next interrupt arrives rather than
// the expensive loop
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    // unsafe: relies on fact that a special QEMU device is attached to the I/O port w/ address 0xf4
    // Provides exiting qemu without a 'proper' shutdown
    unsafe {
        // port type defined as u32 due to qemu iosize option being set to 4B
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// Executes all functions annotated with: #[test_case]
// and exits qemu
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

// The panic handler to be used during testing
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

