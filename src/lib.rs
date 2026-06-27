#![no_std]
#![no_main]

use core::panic::PanicInfo;
pub mod vga_writer;

/// This is your OS entry point. 
/// `extern "C"` forces the compiler to use the standard C calling convention, 
/// and `#[no_mangle]` keeps the name exactly `rust_main` so assembly can find it.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    let mut vga_writer = vga_writer::VGAWriter::init();

    vga_writer.line_o = 2;//Avoid overlap with 2 previous line of the text print by boot.asm and long_mode.asm
    vga_writer.print("Hello world!");

    loop {}
}

/// A panic handler is mandatory when working with #![no_std]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}