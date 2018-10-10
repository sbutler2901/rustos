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

// Unit tests run on host machine, therefore std lib available
#[cfg(test)]
extern crate std;
#[cfg(test)]
extern crate array_init;

#[macro_use]        // says to also import macros from module
mod vga_buffer;

use core::panic::PanicInfo;


// The function expected in bootloader assembly for the start of the program
#[cfg(not(test))]
#[no_mangle]        // ensures function name is not mangled for usage by bootloader
pub extern "C" fn _start() -> ! {
    println!("Helo World{}", "!");
    loop {}
}

// Defines the method to use in case of a panic
#[cfg(not(test))]       // only compile when test flag is not set
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
