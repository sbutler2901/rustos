#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate rust_os;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

use rust_os::{exit_qemu, hlt_loop, QemuExitCode};
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};

pub const SYS_CALL_ID: u8 = 0x80;       // base 10: 128

pub fn init_idt() { IDT.load(); }

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        let sys_call_interrupt_id = usize::from(SYS_CALL_ID);
        idt[sys_call_interrupt_id].set_handler_fn(sys_call_interrupt_handler);

        idt
    };
}

extern "x86-interrupt" fn sys_call_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame
) {
    serial_println!("ok");

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[cfg(not(test))]
#[no_mangle]
#[allow(const_err)]
pub extern "C" fn _start() -> ! {
    rust_os::gdt::init();
    init_idt();

    unsafe { asm!("int 0x80" :::: "intel"); }

    serial_println!("failed");
    serial_println!("No exception occurred");

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");
    serial_println!("{}", info);

    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}
