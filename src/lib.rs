#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::vga_writer::VGAWriter;
pub mod vga_writer;
pub mod interrupts;

/// This is your OS entry point. 
/// `extern "C"` forces the compiler to use the standard C calling convention, 
/// and `#[no_mangle]` keeps the name exactly `rust_main` so assembly can find it.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(mbi_ptr: usize) -> ! {
    unsafe {
        interrupts::init_idt();

        core::arch::asm!("int3");
    }

    let mut vga_writer = vga_writer::VGAWriter::init();
    
    vga_writer.set_color(vga_writer::VGAOutColor::Green, vga_writer::VGAOutColor::Black);
    
    vga_writer.line_o = 2;//start from line no.3 to avoid overlap with 2 previous line of the text print by boot.asm and long_mode.asm

    write!(vga_writer, "hello world! using vga buffer");

    loop {}
}

/// A panic handler is mandatory when working with #![no_std]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}