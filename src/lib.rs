#![no_std]
#![no_main]

use core::panic::PanicInfo;
pub mod vga_writer;

/// This is your OS entry point. 
/// `extern "C"` forces the compiler to use the standard C calling convention, 
/// and `#[no_mangle]` keeps the name exactly `rust_main` so assembly can find it.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    // Pointer to the VGA text buffer 
    // let vga_buffer = 0xb8000 as *mut u8;

    // unsafe {
        // *vga_buffer.offset(320) = b'R';
        // *vga_buffer.offset(321) = 0x9f;
        // *vga_buffer.offset(322) = b'U';
        // *vga_buffer.offset(323) = 0x9f;        
        // *vga_buffer.offset(324) = b'S';
        // *vga_buffer.offset(325) = 0x9f;        
        // *vga_buffer.offset(326) = b'T';
        // *vga_buffer.offset(327) = 0x9f;

        // *vga_buffer.offset(480) = b'H';
        // *vga_buffer.offset(481) = 0x9f;
        // *vga_buffer.offset(482) = b'A';
        // *vga_buffer.offset(483) = 0x9f;        
        // *vga_buffer.offset(484) = b'I';
        // *vga_buffer.offset(485) = 0x9f;        
        // *vga_buffer.offset(486) = b' ';
        // *vga_buffer.offset(487) = 0x9f;
        // *vga_buffer.offset(488) = b'G';
        // *vga_buffer.offset(489) = 0x9f;
        // *vga_buffer.offset(490) = b'A';
        // *vga_buffer.offset(491) = 0x9f;
        // *vga_buffer.offset(492) = b'Y';
        // *vga_buffer.offset(493) = 0x9f;
    // }

    let mut vga_writer = vga_writer::VGAWriter::init();

    vga_writer.line_o = 2;
    vga_writer.print("Hello world!");

    loop {}
}

/// A panic handler is mandatory when working with #![no_std]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}