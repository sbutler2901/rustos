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

// use core::panic::PanicInfo;

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

// unsafe: relies on fact that a special QEMU device is attached to the I/O port w/ address 0xf4
// Provides exiting qemu without a 'proper' shutdown
pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    // port type defined as u32 due to qemu iosize option being set to 4B
    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}
