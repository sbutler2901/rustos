use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, PageFaultErrorCode, InterruptStackFrame };

pub const PIC_1_OFFSET: u8 = 32;    // offset interrupts to 32 (where CPU exceptions end)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;  // start secondary PIC exceptions after 8 for first
pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;    // timer interrupt (0 + offset)
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;     // keyboard interrupt
pub const SYS_CALL_ID: u8 = 0x80;       // base 10: 128

// unsafe: wrong offset could cause undefined behavior
// Mutex provides safe mutable access (when lock method used)
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Initialize the CPUs IDT
pub fn init_idt() {
    IDT.load();
}

// Static IDT for CPU to reference during exceptions
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        use gdt;

        let mut idt = InterruptDescriptorTable::new();

        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        idt.overflow.set_handler_fn(overflow_interrupt_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);

        // unsafe: caller must ensure that used stack index is valid
        // and not already used for another exception
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                // tells CPU to switch to this stack before invoking handler
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
         }

        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        idt.virtualization.set_handler_fn(virtualization_handler);
        idt.security_exception.set_handler_fn(security_exception_handler);

        // PIC interrupts
        let timer_interrupt_id = usize::from(TIMER_INTERRUPT_ID);
        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);

        let keyboard_interrupt_id = usize::from(KEYBOARD_INTERRUPT_ID);
        idt[keyboard_interrupt_id].set_handler_fn(keyboard_interrupt_handler);

        // Sys call interrupt
        let sys_call_interrupt_id = usize::from(SYS_CALL_ID);
        idt[sys_call_interrupt_id].set_handler_fn(sys_call_interrupt_handler);
        idt
    };
}

// Exceptions
// Note: Updates to these functions should also be made in their corresponding test-exception-*.rs files

// Faults: These can be corrected and the program may continue as if nothing happened.
// Traps: Traps are reported immediately after the execution of the trapping instruction.
// Aborts: Some severe unrecoverable error.

/// Fault: occurs when dividing any number by 0
extern "x86-interrupt" fn divide_error_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault/Trip: debug exceptions
extern "x86-interrupt" fn debug_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
    hlt_loop();
}

/// Trap: Handler for breakpoint exception
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Interrupt: Handler for non maskable interrupts
extern "x86-interrupt" fn non_maskable_interrupt_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: NON-MASKABLE\n{:#?}", stack_frame);
    hlt_loop();
}

/// Trip: Handler for
extern "x86-interrupt" fn overflow_interrupt_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles bound range exceeded exception
extern "x86-interrupt" fn bound_range_exceeded_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles invalid opcode exception
extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles device not available exception
extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
    hlt_loop();
}

// Occurs when a cpu exception is not caught.
// if not implemented and needed a triple fault will occur
// this results (mostly) in a complete reboot
/// About: Handler for double fault exception
extern "x86-interrupt" fn double_fault_handler(
    // error_code by definition always 0
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) -> ! {
    use hlt_loop;

    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles invalid tss exception
extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles segment not present exception
extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: handles stack segment fault exception
extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: general exception errors
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;
//    println!("Error code: {}", error_code);
    println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler for page fault exception
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: PageFaultErrorCode
) {
    use hlt_loop;
    // automatically set on page fault to accessed virtual address that caused page fault
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("{:#?}", stack_frame);
    println!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler for x87 floating point exception
extern "x86-interrupt" fn x87_floating_point_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: X87 FLOATING POINT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler for alignment check exception
extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: ALIGNMENT CHECK\n{:#?}", stack_frame);
    hlt_loop();
}

/// Abort: Handler for machine check exception
extern "x86-interrupt" fn machine_check_handler(
    stack_frame: &mut InterruptStackFrame
) -> ! {
    use hlt_loop;

    println!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler for simd floating point exception
extern "x86-interrupt" fn simd_floating_point_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler virtualization exception
extern "x86-interrupt" fn virtualization_handler(
    stack_frame: &mut InterruptStackFrame
) {
    use hlt_loop;

    println!("EXCEPTION: VIRTUALIZATION\n{:#?}", stack_frame);
    hlt_loop();
}

/// Fault: Handler security exception
extern "x86-interrupt" fn security_exception_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: SECURITY EXCEPTION\n{:#?}", stack_frame);
    hlt_loop();
}

// Hardware Interrupts

/// Handler to timer interrupts
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame
) {
//    print!(".");
    // PIC waits for EOI signal notifying ready for next interrupt
    // unsafe: incorrect interrupt vector number could result in deleting unsent interrupt
    // causing system to hang
    unsafe { PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID) }
}

/// Handler for keyboard interrupts
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame
) {
    use x86_64::instructions::port::Port;
    use keyboard;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let key = keyboard::scancode_map(scancode);

    if let Some(key) = key {
        print!("{}", key);
    } /*else {
        // debugging of unmapped scancodes
        print!(" {} ", scancode);
    }*/
    //print!("Exception: breakpoint\n{:#?}", stack_frame);
    unsafe { PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID)}
}

// Software Interrupts

/// Fault: sys call interrupt
extern "x86-interrupt" fn sys_call_interrupt_handler(
    stack_frame: &mut InterruptStackFrame
) {
    println!("EXCEPTION: SYS CALL\n{:#?}", stack_frame);
}
