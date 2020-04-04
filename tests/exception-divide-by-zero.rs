#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

use rust_os::{exit_qemu, hlt_loop, QemuExitCode, serial_println};
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};

pub fn init_idt() { IDT.load(); }

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);

        idt
    };
}

extern "x86-interrupt" fn divide_error_handler(
    _stack_frame: &mut InterruptStackFrame
) {
    serial_println!("ok");

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[no_mangle]
#[allow(const_err)]
pub extern "C" fn _start() -> ! {
    rust_os::gdt::init();
    init_idt();

    unsafe {
        asm!(r"
            mov eax, 0x1
            mov ecx, 0x0
            div ecx"
            :::: "intel"
        );
    }

    serial_println!("failed");
    serial_println!("No exception occurred");

    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
