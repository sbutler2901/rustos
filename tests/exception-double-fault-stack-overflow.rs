#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use rust_os::{exit_qemu, hlt_loop, QemuExitCode, serial_println};
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};

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
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("ok");

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rust_os::gdt::init();
    init_idt();

    // trigger a stack overflow
    stack_overflow();

    serial_println!("failed");
    serial_println!("No exception occurred");

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
}


/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
