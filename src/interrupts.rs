use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame };

pub const PIC_1_OFFSET: u8 = 32;    // offset interrupts to 32 (where CPU exceptions end)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;  // start secondary PIC exceptions after 8 for first
pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;    // timer interrupt (0 + offset)
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;     // keyboard interrupt

// unsafe: wrong offset could cause undefined behavior
// Mutex provides safe mutable access (when lock method used)
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Static IDT for CPU to reference during exceptions
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        use gdt;

        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // unsafe: caller must ensure that used stack index is valid
        // and not already used for another exception
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                // tells CPU to switch to this stack before invoking handler
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
         }

        // PIC interrupts
        let timer_interrupt_id = usize::from(TIMER_INTERRUPT_ID);
        idt[timer_interrupt_id].set_handler_fn(timer_interrupt_handler);

        let keyboard_interrupt_id = usize::from(KEYBOARD_INTERRUPT_ID);
        idt[keyboard_interrupt_id].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// Initialize the CPUs IDT
pub fn init_idt() {
    IDT.load();
}

// Exceptions

// Occurs when a cpu exception is not caught.
// if not implemented and needed a triple fault will occur
// this results (mostly) in a complete reboot
/// Handler for double fault exception
extern "x86-interrupt" fn double_fault_handler(
    // error_code by definition always 0
    stack_frame: &mut ExceptionStackFrame, _error_code: u64
) {
    use hlt_loop;

    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    hlt_loop();
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
    unsafe { PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID) }
}

/// Handler for keyboard interrupts
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut ExceptionStackFrame
) {
    use x86_64::instructions::port::Port;
    use keyboard;

    let port = Port::new(0x60);
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