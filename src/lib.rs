#![no_std]

#[macro_use]
extern crate lazy_static;
extern crate bootloader_precompiled;
extern crate volatile;
extern crate spin;
extern crate uart_16550;    // as serial interface for port mapped I/O
extern crate x86_64;

// Unit tests run on host machine, therefore std lib available
#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;

pub mod vga_buffer;
pub mod serial;
pub mod gdt;

// unsafe: relies on fact that a special QEMU device is attached to the I/O port w/ address 0xf4
// Provides exiting qemu without a 'proper' shutdown
pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    // port type defined as u32 due to qemu iosize option being set to 4B
    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}
