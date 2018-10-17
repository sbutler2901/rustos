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
use x86_64::structures::idt::{ExceptionStackFrame, InterruptDescriptorTable};

pub fn init_idt() { IDT.load(); }

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    serial_println!("ok");

    unsafe {
        exit_qemu();
    }
    hlt_loop();
}

#[cfg(not(test))]
#[no_mangle]
#[allow(unconditional_recursion)]
pub extern "C" fn _start() -> ! {
    rust_os::gdt::init();
    init_idt();

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // trigger a stack overflow
    stack_overflow();

    serial_println!("failed");
    serial_println!("No exception occured");

    unsafe {
        exit_qemu();
    }
    hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);

    unsafe {
        exit_qemu();
    }
    hlt_loop();
}
