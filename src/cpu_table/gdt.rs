use core::{ptr::addr_of, ptr::addr_of_mut};

//--GDT setup-------------------------------------------------------------------
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub resr_1: u32,
    pub pst: [u64;3],
    pub resr_2: u64,
    pub ist: [u64;7],
    pub resr_3: u64,
    pub resr_4: u16,
    pub io_map_base: u16,
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
pub struct GDTTable {
    pub null_des:GDTEntry,
    pub kernel_code:GDTEntry,
    pub kernel_data:GDTEntry,
    pub user_code:GDTEntry,
    pub user_data:GDTEntry,
    pub tss:TaskStateSegmentGDTEntry,
}

pub static mut TSS:TaskStateSegment = TaskStateSegment::null();
pub static mut TSS_REF:&mut TaskStateSegment = unsafe { &mut *addr_of_mut!(TSS) };

pub static mut GDT_TABLE:GDTTable = GDTTable {
    null_des:GDTEntry::null_entry(),
    kernel_code:GDTEntry::kernel_code_entry(),
    kernel_data:GDTEntry::kernel_data_entry(),
    user_code:GDTEntry::user_code_entry(),
    user_data:GDTEntry::user_data_entry(),
    tss:TaskStateSegmentGDTEntry::null(),
};

pub unsafe fn reload_segments() {
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

pub unsafe fn init_gdt() {
    unsafe {
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

pub unsafe fn load_tss() {
    unsafe {
        core::arch::asm!(
            "ltr {sel:x}",
            sel = in(reg) 0x28u16,  // selector 0x28 = TSS (index 5, bytes 40..55)
            options(nostack)
        );
    }
}