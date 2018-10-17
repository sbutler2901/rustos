#![no_std]  // dont linke Rust stdlib
// only enable no_main macro in non-test mode
// which prevents a main from being injected since this is a kernel
#![cfg_attr(not(test), no_main)]
// silence certain warnings when testing is being performed
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(abi_x86_interrupt)]  // enable usage of unstable x86-interrupt calling convention

#[macro_use]
extern crate rust_os;
#[macro_use]
extern crate lazy_static;
extern crate x86_64;

use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame };
use rust_os::{gdt, interrupts};

// The function expected in linker for the start of the program
#[cfg(not(test))]
#[no_mangle]        // ensures function name is not mangled for usage by bootloader
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    serial_println!("Hello Host{}", "!");

    gdt::init();    // load GDT
    init_idt();     // load IDT

    // Initialize PICs for hardware interrupts
    // unsafe: possible undefined behavior if PIC misconfigured
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();     // enables external interrupts

    println!("It did not crash!");
    rust_os::hlt_loop();
}

// Defines the method to use in case of a panic
#[cfg(not(test))]       // only compile when test flag is not set
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rust_os::hlt_loop();
}

// Initialize the CPUs IDT
pub fn init_idt() {
    IDT.load();
}

// Static IDT for CPU to reference during exceptions
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // unsafe: caller must ensure that used stack index is valid
        // and not already used for another exception
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                // tells CPU to switch to this stack before invoking handler
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
         }

        let timer_interrupt_id = usize::from(interrupts::TIMER_INTERRUPT_ID);
        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);

        let keyboard_interrupt_id = usize::from(interrupts::KEYBOARD_INTERRUPT_ID);
        idt[keyboard_interrupt_id].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// Occurs when a cpu exception is not caught.
// if not implemented and needed a triple fault will occur
// this results (mostly) in a complete reboot
/// Handler for double fault exception
extern "x86-interrupt" fn double_fault_handler(
    // error_code by definition always 0
    stack_frame: &mut ExceptionStackFrame, _error_code: u64
) {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    rust_os::hlt_loop();
}

/// Handler for breakpoint exception
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut ExceptionStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler to timer interrupts
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut ExceptionStackFrame
) {
//    print!(".");
    // PIC waits for EOI signal notifying ready for next interrupt
    // unsafe: incorrect interrupt vector number could result in deleting unsent interrupt
    // causing system to hang
    unsafe { interrupts::PICS.lock().notify_end_of_interrupt(interrupts::TIMER_INTERRUPT_ID) }
}

/// Handler for keyboard interrupts
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut ExceptionStackFrame
) {
    use x86_64::instructions::port::Port;

    let port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let key = scancode_map(scancode);

    if let Some(key) = key {
//        if (SHIFT_ACTIVE && key.is_alphabetic()) {
//            print!("{}", key.to_uppercase());
//        }
        print!("{}", key);
    } else {
        print!("{}", scancode);
    }
    //print!("Exception: breakpoint\n{:#?}", stack_frame);
    unsafe { interrupts::PICS.lock().notify_end_of_interrupt(interrupts::KEYBOARD_INTERRUPT_ID)}
}

/// PS/2 Scancode Set 1 mapping
fn scancode_map(scancode: u8) -> Option<char> {
//    let mut SHIFT_ACTIVE = false;
    let key = match scancode {
        0x01 => None,       // escape
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        0x0E => None,       // backspace
        0x0F => None,       // tab
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => None,       // enter
        0x1D => None,       // left ctrl
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),
        0x2A => {
//            SHIFT_ACTIVE = true;
            None
        },       // left shift
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x36 => {           // right shift
//            SHIFT_ACTIVE = true;
            None
        },
        0x37 => None,       // (keypad) * pressed
        0x38 => None,       // left alt
        0x39 => Some(' '),
        0x3A => None,       // Capslock pressed
        0xAA => {           // left shift released
//           SHIFT_ACTIVE = false;
            None
        }
        0xB6 => {           // right shift released
//            SHIFT_ACTIVE = false;
            None
        }
        _ => None,
    };
    key
}