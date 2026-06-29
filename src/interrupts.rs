use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::vga_writer;

pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    // vga_writer::VGAWriter::init().println(content);
}