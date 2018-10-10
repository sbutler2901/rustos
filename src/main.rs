#![no_std]
// only enable no_main macro in non-test mode
// which prevents a main from being injected since this is a kernel
#![cfg_attr(not(test), no_main)]
// silence certain warnings when testing is being performed
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

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

#[macro_use]        // says to also import macros from module
mod vga_buffer;
#[macro_use]
mod serial;

use core::panic::PanicInfo;


// The function expected in bootloader assembly for the start of the program
#[cfg(not(test))]
#[no_mangle]        // ensures function name is not mangled for usage by bootloader
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    unsafe { exit_qemu() }
    loop {}
}

// Defines the method to use in case of a panic
#[cfg(not(test))]       // only compile when test flag is not set
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// unsafe: relies on fact that a special QEMU device is attached to the I/O port w/ address 0xf4
pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    // port type defined as u32 due to qemu iosize option being set to 4B
    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}

