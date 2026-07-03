use core::arch::asm;

pub const PIC1_COMMAND:u16 = 0x20;
pub const PIC1_DATA:u16 = 0x21;
pub const PIC2_COMMAND:u16 = 0xa0;
pub const PIC2_DATA:u16 = 0xa1;

pub const PIC_EOI:u8 = 0x20;

const PIC_ICW1_ICW4:u8 = 0x1;
const PIC_ICW1_SINGLE:u8 = 0x2;
const PIC_ICW1_INTERVAL4:u8 = 0x4;
const PIC_ICW1_LEVEL:u8 = 0x8;
const PIC_ICW1_INIT:u8 = 0x10;

const ICW4_8086:u8 = 0x01;
const ICW4_AUTO:u8 = 0x02;
const ICW4_BUF_SLAVE:u8 = 0x08;
const ICW4_BUF_MASTER:u8 = 0x0C;
const ICW4_SFNM:u8 = 0x10;

const PIC_READ_IRR:u8 = 0x0a;
const PIC_READ_ISR:u8 = 0x0b;

const CASCADE_IRQ:u8 = 2;

pub fn outb(port:u16,val: u8) {
    unsafe {
        asm!("outb {v:x}, {p:x}", v = in(reg) val as u16, p = in(reg) port);
    }
}

pub fn inb(port:u16) -> u16 {
    let ret:u16;

    unsafe {
        asm!("inb {ret:x}, {p:x}", ret = out(reg) ret, p = in(reg) port);
    }

    ret
}

pub fn io_wait() {
    outb(0x80, 0);
}

pub fn are_interrupts_enabled() -> bool {
    let flags: u32;

    unsafe {
        asm!( "pushf",
            "pop {fl:e}",
            fl = out(reg) flags
        );
    }

    return (flags & (1 << 9)) != 0;
}

pub fn pic_init(offset_pic_1:u8, offset_pic_2:u8) {
    outb(PIC1_COMMAND, PIC_ICW1_INIT | PIC_ICW1_ICW4);  // starts the initialization sequence (in cascade mode)
	io_wait();
	outb(PIC2_COMMAND, PIC_ICW1_INIT | PIC_ICW1_ICW4);
	io_wait();
	outb(PIC1_DATA, offset_pic_1);                 // ICW2: Master PIC vector offset
	io_wait();
	outb(PIC2_DATA, offset_pic_2);                 // ICW2: Slave PIC vector offset
	io_wait();

    outb(PIC1_DATA, 1 << CASCADE_IRQ);        // ICW3: tell Master PIC that there is a slave PIC at IRQ2
	io_wait();
	outb(PIC2_DATA, 2);                       // ICW3: tell Slave PIC its cascade identity (0000 0010)
	io_wait();
	
    outb(PIC1_DATA, ICW4_8086);               // ICW4: have the PICs use 8086 mode (and not 8080 mode)
	io_wait();
	outb(PIC2_DATA, ICW4_8086);
	io_wait();

	// Unmask both PICs.
	outb(PIC1_DATA, 0);
	outb(PIC2_DATA, 0);
}

pub fn pic_disable() {
    outb(PIC1_DATA, 0xff);
    outb(PIC2_DATA, 0xff);
}

pub fn irq_set_mask(irq_line:u8) {
    let port:u16;
    let value:u8;

    let mut irq_line = irq_line;

    if irq_line < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq_line -= 8;
    }

    value = inb(port) as u8 | ((1 << irq_line));

    outb(port, value);        
}

pub fn irq_clear_mask(irq_line:u8) {
    let port:u16;
    let value:u8;

    let mut irq_line = irq_line;

    if irq_line < 8 {
        port = PIC1_DATA;
    } else {
        port = PIC2_DATA;
        irq_line -= 8;
    }

    value = inb(port) as u8 & (1 << irq_line);

    outb(port, value);        
}

pub fn send_eoi(irq_line:u8) {
    if irq_line >= 8 {
        outb(PIC1_COMMAND, PIC_EOI);
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

pub fn get_irq_reg(ocw3:u8) -> u16 {
    outb(PIC1_COMMAND, ocw3);
    outb(PIC2_COMMAND, ocw3);
    (inb(PIC2_COMMAND) << 8) | inb(PIC1_COMMAND)
}

pub fn get_irr() -> u16 {
    get_irq_reg(PIC_READ_IRR)
}

pub fn get_isr() -> u16 {
    get_irq_reg(PIC_READ_ISR)
}

pub fn enable_interrupt() {
    unsafe {
        asm!("sli");
    }
}

pub fn disable_interrupt() {
    unsafe {
        asm!("cli");
    }
}