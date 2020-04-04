#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use rust_os::{serial_println, exit_qemu, QemuExitCode, hlt_loop};
use lazy_static::lazy_static;
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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_idt();

    test_main();

    exit_qemu(QemuExitCode::Success);
    hlt_loop();
}

#[test_case]
fn test_exception_breakpoint() {
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
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
