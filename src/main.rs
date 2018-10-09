#![no_std]
#![no_main]

use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// MacOS
#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}

// LINUX
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}
