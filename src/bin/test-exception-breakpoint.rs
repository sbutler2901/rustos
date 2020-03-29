#![feature(abi_x86_interrupt)]
#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate rust_os;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

use rust_os::{exit_qemu, hlt_loop};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};

// can safely concurrently modifies because of its all operations are atomic
// used to verify breakpoint_handler is only called once
static BREAKPOINT_HANDLER_CALLED: AtomicUsize = AtomicUsize::new(0);

pub fn init_idt() { IDT.load(); }

// start: Dup from main
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: &mut InterruptStackFrame) {
    // Ordering parameter specifies desired guarantees of the atomic operations
    // SeqCst: "sequential consistent" -> gives strongest guarantees
    BREAKPOINT_HANDLER_CALLED.fetch_add(1, Ordering::SeqCst);
}
// end


#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_idt();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    match BREAKPOINT_HANDLER_CALLED.load(Ordering::SeqCst) {
        1 => serial_println!("ok"),
        0 => {
            serial_println!("failed");
            serial_println!("Breakpoint handler was not called.");
        }
        other => {
            serial_println!("failed");
            serial_println!("Breakpoint handler was called {} times", other);
        }
    }

    unsafe { exit_qemu(); }
    hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);

    unsafe { exit_qemu(); }
    hlt_loop();
}
