#![no_std]
#![no_main]

#[macro_use]
extern crate lazy_static;
extern crate bootloader_precompiled;
extern crate volatile;
extern crate spin;

#[macro_use]        // says to also import macros from module
mod vga_buffer;

use core::panic::PanicInfo;


#[no_mangle]        // ensures function name is not mangled for usage by bootloader
pub extern "C" fn _start() -> ! {
    println!("Helo World{}", "!");
    loop {}
}

// Defines the method to use in case of a panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
