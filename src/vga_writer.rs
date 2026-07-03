use core::ptr::addr_of_mut;

const COL_SIZE:isize = 80;
const ROW_SIZE:isize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VGAOutColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Print library for HLQ OS
/// ```
///use core::fmt::Write;
/// 
///let mut vga_writer = vga_writer::VGAWriter::init();
///
/// write!(vga_writer, "HELLO {}", 322);
/// ```
#[derive(Debug)]
pub struct VGAWriter {
    vga_addr: *mut u8,
    pub line_char_o: isize,//index of currect column
    pub line_o: isize,//index of current row (line)
    pub color: u8,
    pub clear_on_overlow:bool,
}

pub static mut GL_VGA_WT:VGAWriter = VGAWriter {
    vga_addr: 0xb8000 as *mut u8,
    line_char_o: 0,
    line_o: 0,
    color: 0x2f,//Light green
    clear_on_overlow:true,
};

///```no_run
/// //Change color of global VGAwiter
/// vga_writer::GL_VGA_WT_REF.set_color(vga_writer::VGAOutColor::Green, vga_writer::VGAOutColor::Black);
///        
/// vga_writer::GL_VGA_WT_REF.line_o = 2;//Change line
/// ```
pub const GL_VGA_WT_REF:&mut VGAWriter = unsafe {&mut *addr_of_mut!(crate::vga_writer::GL_VGA_WT)};

impl VGAWriter {
    ///### create new instance of VGAWriter
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print("Hello World");
    /// //Expected output:
    /// //Hello World
    /// ```
    pub fn init() -> VGAWriter {
        VGAWriter {
            vga_addr: 0xb8000 as *mut u8,
            line_char_o: 0,
            line_o: 0,
            color: 0x2f,//Light green
            clear_on_overlow:true,
        }
    }

    ///### Set text color
    /// 
    ///### 8 bit input
    /// 
    /// first 4 bit define text color (| Bright | Red | Green | Blue |)
    /// 
    /// second 4 bit define background color (| Bright | Red | Green | Blue |)
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// //0x2f -> 0010(second 4 is green) 1111(first 4 bit mean white)
    /// vga.set_color(0x2f);//Set background color green, white text
    /// ```
    pub fn set_color_hex(&mut self, color:u8) {
        self.color = color;
    }

    ///### Set text color
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// //0x2f -> 0010(second 4 is green) 1111(first 4 bit mean white)
    /// vga.set_color(VGAOutColor::White, VGAOutColor::Green);//Set background color green, white text
    /// ```
    pub fn set_color(&mut self, text_color: VGAOutColor, background_color: VGAOutColor) {
        self.color = ((background_color as u8) << 4) | text_color as u8;

        //how it work:
        //0000(4 bit bg color) << 4 = (4 bit bg color)0000 //Bit shift
        // (4 bit bg color)0000 | 0000(4 bit text color) = (4 bit bg color)(4 bit text color) //OR operation act like merge
    }

    /// ### Break line
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print_char("A");
    /// 
    /// vga.new_line();
    /// 
    /// vga.print_char("A");
    /// 
    /// //Expected output:
    /// //A
    /// //A
    /// ```
    pub fn new_line(&mut self) {
        self.line_o += 1;//add new line
        self.line_char_o = 0;
    }

    /// ### Print a a warning
    pub fn warn(&mut self, content:&str) {
        let backup = self.color;

        self.set_color(VGAOutColor::Red, VGAOutColor::Yellow);
        self.println(content);

        self.color = backup;
    }

    /// ### Print a a warning on start of cmd
    pub fn warn_top(&mut self, content:&str) {
        let backup = (self.color, self.line_o, self.line_char_o);

        self.line_o = 0;
        self.line_char_o = 0;
        
        self.set_color(VGAOutColor::Red, VGAOutColor::Yellow);
        self.println(content);

        self.color = backup.0;
        self.line_o = backup.1;
        self.line_char_o = backup.2;
    }

    /// ### Print a single character
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print_char("A");
    /// //Expected output:
    /// //A
    /// ```
    pub fn print_char(&mut self, char:u8) -> bool {
        if self.line_char_o >= COL_SIZE {
            self.line_o += self.line_char_o / COL_SIZE;

            self.line_char_o = self.line_char_o % COL_SIZE;
        }

        if self.line_o >= ROW_SIZE {
            if self.clear_on_overlow {
                self.clear();
            } else {
                self.warn_top("Overflow");
                return false;
            }
        }

        let c = match char {
            0x20..=0x7e => char, //Only print Ascii char
            _ => 0xfe, //If not ascii char then print "■"
        };

        let offset = (COL_SIZE * self.line_o + self.line_char_o) * 2;//Mapping to memory address offset
        
        unsafe {
            *self.vga_addr.offset(offset) = c;
            *self.vga_addr.offset(offset + 1) = self.color;
        }

        self.line_char_o += 1;

        return true;
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print("Hello World");
    /// //Expected output:
    /// //Hello World
    /// ```
    pub fn print(&mut self, content: &str) {
        for s in content.bytes() {
            match s {
                b'\n' => self.new_line(),
                s => if !self.print_char(s) {
                    return;
                },
            }
        }
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.println("Hello");
    /// vga.println("World");
    /// //Expected output:
    /// //Hello 
    /// //World
    /// ```
    pub fn println(&mut self, content: &str) {
        self.print(content);

        self.new_line();
    }

    ///### Clear cmd
    pub fn clear(&mut self) {
        self.line_o = 0;
        self.line_char_o = 0;

        for c in 0..(ROW_SIZE * COL_SIZE) {
            unsafe {
                *self.vga_addr.offset(c * 2) = 0x0;
                *self.vga_addr.offset(c * 2 + 1) = 0x0;
            }
        }
    }

    ///### Clear specific line cmd
    pub fn clear_line(&mut self, line:isize) {
        self.line_o = 0;
        self.line_char_o = 0;

        let line_of = COL_SIZE * line;

        for c in 0..COL_SIZE {
            unsafe {
                *self.vga_addr.offset((line_of + c) * 2) = 0x0;
                *self.vga_addr.offset((line_of + c) * 2 + 1) = 0x0;
            }
        }
    }
}

use core::fmt;

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
        Ok(())
    }
}

#[macro_export]
///
/// ```no_run
/// vga_print!("pi = {}", 3.14);
/// ```
macro_rules! vga_print {
    ($($arg:tt)*) => {
        unsafe {
            use core::fmt::Write;
            use core::ptr::addr_of_mut;
            let w = &mut *addr_of_mut!($crate::vga_writer::GL_VGA_WT);
            let _ = w.write_fmt(core::format_args!($($arg)*));
        }
    };
}

#[macro_export]
///
/// ```no_run
/// vga_println!("pi = {}", 3.14);
/// vga_println!("pi = {}", 3.14);
/// //Result:
/// //3.14
/// //3.14
/// ```
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", core::format_args!($($arg)*)));
}