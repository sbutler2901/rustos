use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::gdt::SegmentSelector;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// Used to add selectors to the static GDT
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    // load the actual GDT: references the actual GDT at
    // index 0 of the static GDT tuple
    GDT.0.load();

    // unsafe because might be possible to break memory safety
    // by loading invalid selectors.
    unsafe {
        // reload the code segment register with the new one in the GDT tuples index 1
        set_cs(GDT.1.code_selector);
        // tell CPU to use TSS in the GDT tuples index 1
        load_tss(GDT.1.tss_selector);
    }
}

lazy_static! {
    // create a new GDT with a kernel code segment and our TSS segment
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

lazy_static! {
    // TSS: holds two stack tables: (Interrupt Stack Table & privilege stack table)
    // the PST is used by CPU when privilege level changes (exception while CPU
    // in User mode (level 3) and switches to kernel mode (level 0).
    // In this case CPU would switch to the 0th stack in PST since 0 is the
    // target privilege level
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // define the Interrupt Stack Table's 0th index (of 7 total)
        // for the double fault handler. This is done to prevent the
        // the double fault exception handler from being swapped out,
        // or unavailable cause a triple fault! The CPU automatically
        // switches the stack for you when the interrupt occurs.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // create a 4KB stack for double fault
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Store the top (higher numbered address) of the stack
            // since x86 stacks grow downward into lower memory address.
            // No memory management created yet, so no proper way to allocate a
            // new stack. Using a simple array for storage now. Unsafe required because
            // compiler can't guarantee race freedom when static mut is referenced.
            // Must be mutable static else bootload will map to read-only page.
            // No guard page protecting against stack overflow, so no stack intensive tasks.
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}
