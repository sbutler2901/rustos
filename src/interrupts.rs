use pic8259_simple::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;    // offset interrupts to 32 (where CPU exceptions end)
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;  // start secondary PIC exceptions after 8 for first
pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;    // timer interrupt (0 + offset)
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;     // keyboard interrupt

// unsafe: wrong offset could cause undefined behavior
// Mutex provides safe mutable access (when lock method used)
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });