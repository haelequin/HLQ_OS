const COL_SIZE:isize = 80;
const ROW_SIZE:isize = 25;

pub struct VGAWriter {
    pub vga_addr: *mut u8,
    pub line_char_o: isize,//index of currect column
    pub line_o: isize,//index of current row (line)
    pub color: u8,
}

impl VGAWriter {
    ///### create new instance of VGAWriter
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
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
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// //0x2f -> 0010(second 4 is green) 1111(first 4 bit mean white)
    /// vga.set_color(0x2f);//Set background color green, white text
    /// ```
    pub fn set_color(&mut self, color:u8) {
        self.color = color;
    }

    /// ### Break line
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
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
    }

    /// ### Print a single character
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print_char("A");
    /// //Expected output:
    /// //A
    /// ```
    pub fn print_char(&mut self, c:u8) {
        let offset = (COL_SIZE * self.line_o + self.line_char_o) * 2;
        
        unsafe {
            *self.vga_addr.offset(offset) = c;
            *self.vga_addr.offset(offset + 1) = self.color;
        }

        self.line_char_o += 1;
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print("Hello World");
    /// //Expected output:
    /// //Hello World
    /// ```
    pub fn print(&mut self, content: &str) {
        for s in content.bytes() {
            self.print_char(s);
        }
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.println("Hello");
    /// vga.println("World");
    /// //Expected output:
    /// //Hello 
    /// //World
    /// ```
    pub fn println(&mut self, content: &str) {
        for s in content.bytes() {
            self.print_char(s);
        }

        self.new_line();
    }
}