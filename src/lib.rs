#![no_std]
#![feature(abi_x86_interrupt)]  // enable usage of unstable x86-interrupt calling convention

#[macro_use]
extern crate lazy_static;
extern crate bootloader;
extern crate volatile;
extern crate spin;
extern crate uart_16550;    // as serial interface for port mapped I/O
extern crate x86_64;
extern crate pic8259_simple;

// Unit tests run on host machine, therefore std lib available
#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;

#[macro_use]
pub mod vga_buffer;
pub mod serial;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;

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
