use core::ptr::addr_of;
use core::ptr::addr_of_mut;
use crate::vga_println;
use core::arch::{asm,global_asm}; 

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


global_asm!(
    r#"
    .macro exception_stub_no_error name, handler
    .global \name
    \name:
        push 0                    # Dummy error code
        push rax; push rbx; push rcx; push rdx; push rsi; push rdi; push rbp
        push r8;  push r9;  push r10; push r11; push r12; push r13; push r14; push r15
        mov rdi, rsp              # Pass pointer to stack frame as 1st argument (RDI)
        call \handler
        pop r15;  pop r14;  pop r13;  pop r12;  pop r11;  pop r10;  pop r9;  pop r8
        pop rbp;  pop rdi;  pop rsi;  pop rdx;  pop rcx;  pop rbx;  pop rax
        add rsp, 8                # Clean up dummy error code
        iretq                     # Return from interrupt
    .endm

    .macro exception_stub_with_error name, handler
    .global \name
    \name:
        # Error code is already pushed by CPU here
        push rax; push rbx; push rcx; push rdx; push rsi; push rdi; push rbp
        push r8;  push r9;  push r10; push r11; push r12; push r13; push r14; push r15
        mov rdi, rsp              # Pass pointer to stack frame as 1st argument (RDI)
        call \handler
        pop r15;  pop r14;  pop r13;  pop r12;  pop r11;  pop r10;  pop r9;  pop r8
        pop rbp;  pop rdi;  pop rsi;  pop rdx;  pop rcx;  pop rbx;  pop rax
        add rsp, 8                # Clean up CPU error code
        iretq                     # Return from interrupt
    .endm

    # Link the raw ASM stubs to our clean Rust handlers
    exception_stub_no_error   asm_handler_de, rust_handler_de
    exception_stub_no_error   asm_handler_bp, rust_handler_bp
    # exception_stub_with_error asm_handler_of, rust_handler_of
    exception_stub_with_error asm_handler_df, rust_handler_df
    exception_stub_with_error asm_handler_gp, rust_handler_gp
    exception_stub_with_error asm_handler_pf, rust_handler_pf
    "#
);

// Declare the assembly stubs so we can pass them to our IDT
unsafe extern "C" {
    fn asm_handler_de();
    fn asm_handler_bp();
    // fn asm_handler_of();
    fn asm_handler_df();
    fn asm_handler_gp();
    fn asm_handler_pf();
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_de(frame: &ExceptionStackFrame) {
    vga_println!("--- EXCEPTION: DIVIDE BY ZERO (#DE) ---");
    vga_println!("Instruction Pointer (RIP): 0x{}", frame.rip);
    vga_println!("Stack Pointer (RSP):       0x{}", frame.rsp);
    vga_println!("RAX: 0x{} | RBX: 0x{}", frame.rax, frame.rbx);
    // vga_println!("Full stack frame: {:#?} ", frame);
    loop {} // Diverge since we can't safely resume a divide-by-zero easily
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_bp(frame: &ExceptionStackFrame) {
    vga_println!("--- BREAKPOINT (#BP) ---");
    vga_println!("Resuming execution after RIP: 0x{}", frame.rip);
    // Breakpoints are traps, so we can return normally! The stub handles `iretq`.
}

// #[unsafe(no_mangle)]
// pub extern "C" fn rust_handler_of(frame: &ExceptionStackFrame) {
//     vga_println!("--- STACKOVERFLOW(#OF) ---");
// }

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_df(frame: &ExceptionStackFrame) {
    vga_println!("--- DOUBLE FAULT (#DF) ---");
    loop {} // Diverge since we can't safely resume a divide-by-zero easily
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_gp(frame: &ExceptionStackFrame) {
    vga_println!("--- GENERAL PROTECTION FAULT (#GP) ---");
    vga_println!("Error Code:                0x{}", frame.error_code);
    vga_println!("Failing Instruction (RIP): 0x{}", frame.rip);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_pf(frame: &ExceptionStackFrame) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2);
    }
    vga_println!("--- PAGE FAULT (#PF) ---");
    vga_println!("Faulting Memory Address:   0x{}", cr2); // CR2 contains the exact address that triggered the fault
    vga_println!("Error Code Bits:           0x{}", frame.error_code);
    vga_println!("Failing Instruction (RIP): 0x{}", frame.rip);
    loop {}
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
struct IdtEntry {
    offset_low:  u16,   // handler bits [15:0]
    selector:    u16,   // code segment selector
    ist:         u8,    // bits[2:0] = IST index, rest = 0
    type_attr:   u8,    // P | DPL | gate type
    offset_mid:  u16,   // handler bits [31:16]
    offset_high: u32,   // handler bits [63:32]
    _reserved:   u32,
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

// -- The table itself: 256 entries --------------------------------------------
#[repr(C, align(16))]
struct Idt([IdtEntry; 256]);

static mut IDT: Idt = Idt([IdtEntry::missing(); 256]);

// -- Install entries and load --------------------------------------------------
pub unsafe fn init_idt() {
    unsafe {
        // CPU exceptions (vectors 0–31)
        IDT.0[0]  = IdtEntry::new_interrupt(asm_handler_de); 
        IDT.0[3]  = IdtEntry::new_trap(asm_handler_bp);
        // IDT.0[4] = IdtEntry::new_interrupt(asm_handler_of);      
        // IDT.0[4].ist = 1;      
        IDT.0[8] = IdtEntry::new_interrupt(asm_handler_df); 
        IDT.0[8].ist = 1;
        IDT.0[13] = IdtEntry::new_interrupt(asm_handler_gp); 
        IDT.0[14] = IdtEntry::new_interrupt(asm_handler_pf);
        IDT.0[14].ist = 2;
    }

    let descriptor = IdtDescriptor {
        limit: (core::mem::size_of::<Idt>() - 1) as u16,
        base: unsafe {
                addr_of!(IDT.0) as u64
        },
    };

    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &descriptor,
            options(readonly, nostack, preserves_flags)
        );
    }
}

// -- IDTR (what lidt actually receives) ---------------------------------------
#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base:  u64,
}

pub unsafe fn trigger_breakpoint() {
    unsafe {
        asm!("int3");
    }
}

pub unsafe fn trigger_pagefault() {
    unsafe {
        asm!("mov eax, [0xffffffffffffffff]");
    }
}

pub unsafe fn trigger_de() {
    unsafe {
        asm!("mov rax, 100");//move 100 to rax register
        asm!("mov rbx, 0");//move 0 to rbx register
        asm!("div rbx");//divide rax by rbx => 100 / 0 => divide by 0
    }
}

///Only work in debug mode
pub fn stack_overflow() {
    stack_overflow();
}

//--GDT setup-------------------------------------------------------------------
#[repr(C, packed)]
struct TaskStateSegment {
    resr_1: u32,
    pst: [u64;3],
    resr_2: u64,
    ist: [u64;7],
    resr_3: u64,
    resr_4: u16,
    io_map_base: u16,
}

impl TaskStateSegment {
    const fn null() -> Self {
        TaskStateSegment {
            resr_1: 0,
            pst:[0;3],
            resr_2: 0,
            ist: [0;7],
            resr_3: 0,
            resr_4: 0,
            io_map_base: 0,
        }
    }
}

#[repr(C, packed)]
struct TaskStateSegmentGDTEntry {
    limit_1:u16,
    base_1:u16,
    base_2:u8,
    asc_b: u8,
    ///(flags(4 bit) << 4) + limit(4 bit)
    limit_2_flag:u8,
    base_3:u8,
    base_4:u32,
    resr:u32,
}

impl TaskStateSegmentGDTEntry {
    fn new(tss_addr:u64) -> Self {
        let limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u32;
        let base = tss_addr;

        TaskStateSegmentGDTEntry { 
            limit_1: limit as u16, 
            base_1: base as u16, 
            base_2: (base >> 16) as u8, 
            asc_b: 0x89, 
            limit_2_flag: ((limit >> 16) as u8), 
            base_3: (base >> 24) as u8, 
            base_4: (base >> 32) as u32, 
            resr: 0
        }
    }

    const fn null() -> Self {
        TaskStateSegmentGDTEntry { 
            limit_1: 0, 
            base_1: 0, 
            base_2: 0, 
            asc_b: 0, 
            limit_2_flag: 0, 
            base_3: 0, 
            base_4: 0, 
            resr: 0
        }
    } 
}

#[repr(C, packed)]
struct GDTEntry {
    limit_1:u16,
    base_1:u16,
    base_2:u8,
    asc_b: u8,
    ///(flags(4 bit) << 4) + limit(4 bit)
    limit_2_flag:u8,
    base_3:u8,
}

impl GDTEntry {
    const fn null_entry() -> Self {
        GDTEntry {
            limit_1:0,
            base_1:0,
            base_2:0,
            asc_b: 0,
            limit_2_flag:0,
            base_3:0,
        }
    }

    const fn create_entry(flag:u8, limit:u32, asc_b:u8) -> Self {
        GDTEntry {
            limit_1:limit as u16,
            base_1:0,
            base_2:0,
            asc_b: asc_b,
            //(flags(4 bit) << 4) + limit(4 bit)
            limit_2_flag:((flag << 4) as u8) + ((limit >> 16) as u8),
            base_3:0,
        }
    }

    const fn kernel_code_entry() -> Self {
        GDTEntry::create_entry(0xa, 0xfffff, 0x9a)
    }

    const fn kernel_data_entry() -> Self {
        GDTEntry::create_entry(0xc, 0xfffff, 0x92)
    }

    const fn user_code_entry() -> Self {
        GDTEntry::create_entry(0xa, 0xfffff, 0xfa)
    }
    
    const fn user_data_entry() -> Self {
        GDTEntry::create_entry(0xc, 0xfffff, 0xf2)
    }
}

#[repr(C, packed)]
struct GDTPointer {
    limit:u16,
    base:u64,
}

#[repr(C, packed)]
struct GDTTable {
    null_des:GDTEntry,
    kernel_code:GDTEntry,
    kernel_data:GDTEntry,
    user_code:GDTEntry,
    user_data:GDTEntry,
    tss:TaskStateSegmentGDTEntry,
}

static mut TSS:TaskStateSegment = TaskStateSegment::null();

static mut GDT_TABLE:GDTTable = GDTTable {
    null_des:GDTEntry::null_entry(),
    kernel_code:GDTEntry::kernel_code_entry(),
    kernel_data:GDTEntry::kernel_data_entry(),
    user_code:GDTEntry::user_code_entry(),
    user_data:GDTEntry::user_data_entry(),
    tss:TaskStateSegmentGDTEntry::null(),
};

unsafe fn reload_segments() {
    unsafe {
        core::arch::asm!(
            // CS can't be loaded with mov — must do a far return trick
            "push {cs}",            // push new CS selector
            "lea {tmp}, [rip + 2f]",// push return address (label 1)
            "push {tmp}",
            "retfq",                // far return: pops RIP then CS
            "2:",
            // now reload the data segment registers
            "mov ax, {ds}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            cs  = const 0x08u64,    // selector 0x08 = kernel code (index 1)
            ds  = const 0x10u16,    // selector 0x10 = kernel data (index 2)
            tmp = lateout(reg) _,
            options(nostack)
        );
    }
}

static STACK_SIZE:usize = 4096 * 4;
static mut DOUBLEFAULT_STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];
static mut PAGEFAULT_STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];

unsafe fn init_gdt() {
    unsafe {
        TSS.ist[0] = addr_of!(DOUBLEFAULT_STACK) as u64;
        TSS.ist[1] = addr_of!(PAGEFAULT_STACK) as u64;

        GDT_TABLE.tss = TaskStateSegmentGDTEntry::new(addr_of!(TSS) as u64);

        let ptr = GDTPointer {
            limit: (core::mem::size_of::<GDTTable>() - 1) as u16,
            base:  addr_of!(GDT_TABLE) as u64,
        };

        core::arch::asm!(
            "lgdt [{ptr}]",
            ptr = in(reg) &ptr,
            options(nostack)
        );
    }
}

unsafe fn load_tss() {
    unsafe {
        core::arch::asm!(
            "ltr {sel:x}",
            sel = in(reg) 0x28u16,  // selector 0x28 = TSS (index 5, bytes 40..55)
            options(nostack)
        );
    }
}

pub unsafe fn init() {
    unsafe {
        init_idt();
        init_gdt();
        reload_segments();
        load_tss();
    }
}