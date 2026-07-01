#![no_std]
#![no_main]

use core::ptr::addr_of;
use core::fmt::Write;

use crate::vga_writer;

// ── Gate type + attribute byte ──────────────────────────────────────────────
const GATE_PRESENT:    u8 = 1 << 7;   // P bit
const GATE_DPL0:       u8 = 0 << 5;   // ring 0
const GATE_INTERRUPT:  u8 = 0xE;      // 64-bit interrupt gate
const GATE_TRAP:       u8 = 0xF;      // 64-bit trap gate

const KERNEL_CS: u16 = 0x08;          // your GDT code segment selector

// ── 16-byte IDT entry ────────────────────────────────────────────────────────
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

// ── The table itself: 256 entries ────────────────────────────────────────────
#[repr(C, align(16))]
struct Idt([IdtEntry; 256]);

static mut IDT: Idt = Idt([IdtEntry::missing(); 256]);

// ── IDTR (what lidt actually receives) ───────────────────────────────────────
#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base:  u64,
}

// ── Raw exception stubs (must be `extern "C"`, no Rust ABI mangling) ─────────
unsafe extern "C" fn handler_de()  { /* #DE divide-by-zero    */ loop {} }
unsafe extern "C" fn handler_db()  { /* #DB debug             */ loop {} }
unsafe extern "C" fn handler_nmi() { /* NMI                   */ loop {} }

unsafe extern "C" fn handler_bp()  {
    write!(vga_writer::VGAWriter::init(), "break point");
}

unsafe extern "C" fn handler_of()  { /* #OF overflow (trap)   */ loop {} }
unsafe extern "C" fn handler_gp()  { /* #GP general protection*/ loop {} }
unsafe extern "C" fn handler_pf()  { /* #PF page fault        */ loop {} }

// ── Install entries and load ──────────────────────────────────────────────────
pub unsafe fn init_idt() {
    // CPU exceptions (vectors 0–31)
    unsafe {
        IDT.0[0]  = IdtEntry::new_interrupt(handler_de);   // #DE
        IDT.0[1]  = IdtEntry::new_interrupt(handler_db);   // #DB
        IDT.0[2]  = IdtEntry::new_interrupt(handler_nmi);  // NMI
        IDT.0[3]  = IdtEntry::new_trap     (handler_bp);   // #BP  ← trap so RIP advances
        IDT.0[4]  = IdtEntry::new_trap     (handler_of);   // #OF
        IDT.0[13] = IdtEntry::new_interrupt(handler_gp);   // #GP
        IDT.0[14] = IdtEntry::new_interrupt(handler_pf);   // #PF
    }
    // vectors 32–255 → hardware IRQs / syscalls, fill as needed

    let descriptor = IdtDescriptor {
        limit: (core::mem::size_of::<Idt>() - 1) as u16,
        base: unsafe {
                addr_of!(IDT.0) as u64
            },
        };

    core::arch::asm!(
        "lidt [{}]",
        in(reg) &descriptor,
        options(readonly, nostack, preserves_flags)
    );
}