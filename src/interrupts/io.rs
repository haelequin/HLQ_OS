use crate::interrupts::pic_8259_interrupt;

const PIC_OFFSET:u8 = 0x20;//use interrupt vec 32th and above

pub fn irq_init() {
    pic_8259_interrupt::pic_init(PIC_OFFSET, PIC_OFFSET + 8);
}