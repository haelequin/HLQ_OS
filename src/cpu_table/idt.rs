use core::ptr::{addr_of_mut, addr_of};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionStackFrame {
    // Pushed by our assembly stub
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9:  u64, pub r8:  u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    
    // Pushed automatically or manually handled for error codes
    pub error_code: u64,
    
    // Automatically pushed by x86_64 CPU hardware
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionStackFrameNoErr {
    // Pushed by our assembly stub
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9:  u64, pub r8:  u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    
    // Automatically pushed by x86_64 CPU hardware
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

// -- Gate type + attribute byte ----------------------------------------------
const GATE_PRESENT:    u8 = 1 << 7;   // P bit
const GATE_DPL0:       u8 = 0 << 5;   // ring 0
const GATE_INTERRUPT:  u8 = 0xE;      // 64-bit interrupt gate
const GATE_TRAP:       u8 = 0xF;      // 64-bit trap gate

const KERNEL_CS: u16 = 0x08;          // GDT code segment selector

// -- 16-byte IDT entry --------------------------------------------------------
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    pub offset_low:  u16,   // handler bits [15:0]
    pub selector:    u16,   // code segment selector
    pub ist:         u8,    // bits[2:0] = IST index, rest = 0
    pub type_attr:   u8,    // P | DPL | gate type
    pub offset_mid:  u16,   // handler bits [31:16]
    pub offset_high: u32,   // handler bits [63:32]
    pub _reserved:   u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        IdtEntry {
            offset_low:  0,
            selector:    0,
            ist:         0,
            type_attr:   0,   // P=0 → not present
            offset_mid:  0,
            offset_high: 0,
            _reserved:   0,
        }
    }

    fn new(handler: unsafe extern "C" fn(), gate: u8) -> Self {
        let addr = handler as usize;
        IdtEntry {
            offset_low:  (addr & 0xFFFF) as u16,
            selector:    KERNEL_CS,
            ist:         0,
            type_attr:   GATE_PRESENT | GATE_DPL0 | gate,
            offset_mid:  ((addr >> 16) & 0xFFFF) as u16,
            offset_high: ((addr >> 32) & 0xFFFF_FFFF) as u32,
            _reserved:   0,
        }
    }

    fn new_trap(handler: unsafe extern "C" fn()) -> Self {
        Self::new(handler, GATE_TRAP)
    }

    fn new_interrupt(handler: unsafe extern "C" fn()) -> Self {
        Self::new(handler, GATE_INTERRUPT)
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base:  u64,
}


// -- The table itself: 256 entries --------------------------------------------
#[repr(C, align(16))]
pub struct Idt(pub [IdtEntry; 256]);

///INTR mean interrupt
pub enum HandlerType {
    TRAP,
    INTR,
}

#[macro_export]
macro_rules! intr_handler {
    ($fn_h:ident) => {
        unsafe {
            core::arch::asm!(
                // "push rax",//Rust already push rax for us, it take a while to know that:P
                "push rbx; push rcx; push rdx",
                "push rsi; push rdi; push rbp",
                "push r8;  push r9;  push r10; push r11",
                "push r12; push r13; push r14; push r15",
                "mov rdi, rsp",
                "call {h}",
                "pop r15; pop r14; pop r13; pop r12",
                "pop r11; pop r10; pop r9;  pop r8",
                "pop rbp; pop rdi; pop rsi; pop rdx",
                "pop rcx; pop rbx; pop rax",
                "iretq",
                h = sym $fn_h,
            );
        }
    };
}

#[macro_export]
macro_rules! hw_intr_handler {
    ($fn_h:ident) => {
        unsafe {
            core::arch::asm!(
                // "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "call {h}",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "iretq",
                h = sym $fn_h,
            );
        }
    };
}

impl Idt {
    pub fn add_interrupt(&mut self, indx:usize, handler: unsafe extern "C" fn(), h_type:HandlerType, ist:u8) -> bool {
        if indx >= 256 || ist > 7 {
            return false;
        }

        match h_type {
            HandlerType::TRAP => self.0[indx] = IdtEntry::new_trap(handler),
            HandlerType::INTR => self.0[indx] = IdtEntry::new_interrupt(handler),
        }

        self.0[indx].ist = ist;

        true
    }

    pub fn load_idt(&self) {
        let descriptor = IdtDescriptor {
            limit: (core::mem::size_of::<Idt>() - 1) as u16,
            base: addr_of!(self.0) as u64,
        };

        unsafe {
            core::arch::asm!(
                "lidt [{}]",
                in(reg) &descriptor,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

pub static mut IDT: Idt = Idt([IdtEntry::missing(); 256]);

pub const IDT_REF:&mut Idt = unsafe {&mut *addr_of_mut!(IDT)};