#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]  // enable usage of unstable x86-interrupt calling convention
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
pub mod vga_buffer;
pub mod serial;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;

use core::panic::PanicInfo;

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
    use x86_64::instructions::port::Port;

    // unsafe: relies on fact that a special QEMU device is attached to the I/O port w/ address 0xf4
    // Provides exiting qemu without a 'proper' shutdown
    unsafe {
        // port type defined as u32 due to qemu iosize option being set to 4B
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

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
    // test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
