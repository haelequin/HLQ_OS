use core::ptr::addr_of;
use crate::cpu_table::gdt::TSS_REF;
use crate::vga_println;
use core::arch::{asm}; 
use crate::pic_8259_interrupt;
use crate::intr_handler;
use crate::cpu_table::*;

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_de() {
    intr_handler!(de_print);

    extern "C" fn de_print(frame:&idt::ExceptionStackFrameNoErr) {
        vga_println!("--- EXCEPTION: DIVIDE BY ZERO (#DE) ---");
        vga_println!("Instruction Pointer (RIP): 0x{}", frame.rip);
        vga_println!("Stack Pointer (RSP):       0x{}", frame.rsp);
        vga_println!("RAX: 0x{} | RBX: 0x{}", frame.rax, frame.rbx);
        loop {} // Diverge since we can't safely resume a divide-by-zero easily
    }
}

// #[unsafe(no_mangle)]
pub extern "C" fn rust_handler_bp() {
    intr_handler!(bp_print);
    
    extern "C" fn bp_print(frame:&idt::ExceptionStackFrameNoErr) {
        vga_println!("--- BREAKPOINT (#BP) ---");
        vga_println!("Resuming execution after RIP: 0x{}", frame.rip);
    }// Breakpoints are traps, so we can return normally! The stub handles `iretq`.
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_of() {
    vga_println!("--- STACKOVERFLOW(#OF) ---");
    loop {} 
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_df() {
    vga_println!("--- DOUBLE FAULT (#DF) ---");
    loop {} // Diverge since we can't safely resume a divide-by-zero easily
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_gp() {
    intr_handler!(gp_print);

    extern "C" fn gp_print(frame:&idt::ExceptionStackFrame) {
        vga_println!("--- GENERAL PROTECTION FAULT (#GP) ---");
        vga_println!("Error Code:                0x{}", frame.error_code);
        vga_println!("Failing Instruction (RIP): 0x{}", frame.rip);
        
        loop {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_pf() {
    intr_handler!(pf_print);
    
    extern "C" fn pf_print(frame:&idt::ExceptionStackFrame) {
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
}

// -- Install entries and load --------------------------------------------------
pub unsafe fn init_idt() {
    unsafe {
        TSS_REF.ist[0] = addr_of!(DOUBLEFAULT_STACK) as u64;
        TSS_REF.ist[1] = addr_of!(PAGEFAULT_STACK) as u64;
        TSS_REF.ist[2] = addr_of!(OVERFLOW_STACK) as u64;
    }

    idt::IDT_REF.add_interrupt(0, rust_handler_de, idt::HandlerType::INTR, 0);
    idt::IDT_REF.add_interrupt(3, rust_handler_bp, idt::HandlerType::TRAP, 0);
    idt::IDT_REF.add_interrupt(4, rust_handler_of, idt::HandlerType::INTR, 3);
    idt::IDT_REF.add_interrupt(8, rust_handler_df, idt::HandlerType::INTR, 1);
    idt::IDT_REF.add_interrupt(13, rust_handler_gp, idt::HandlerType::INTR, 0);
    idt::IDT_REF.add_interrupt(14, rust_handler_pf, idt::HandlerType::INTR, 2);

    idt::IDT_REF.load_idt();
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
    #![allow(unconditional_recursion)]
    stack_overflow();
}

static STACK_SIZE:usize = 1024;
static mut DOUBLEFAULT_STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];
static mut PAGEFAULT_STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];
static mut OVERFLOW_STACK:[u8;STACK_SIZE] = [0;STACK_SIZE];

pub unsafe fn init() {
    unsafe {
        init_idt();
        gdt::init_gdt();
        gdt::reload_segments();
        gdt::load_tss();
    }
}