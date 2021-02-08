use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{println, gdt};

lazy_static!(
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
);

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(st_fr: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", st_fr);
}

extern "x86-interrupt" fn double_fault_handler(st_fr: &mut InterruptStackFrame, _err_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", st_fr);
}

#[test_case]
fn breakpoint_handling() {
    x86_64::instructions::interrupts::int3();
}