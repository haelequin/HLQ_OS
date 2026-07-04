pub fn load_cr3() -> usize {
    let p4_table;

    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) p4_table);
    }

    p4_table
}

#[repr(C, packed)]
pub struct PageTableEntry {
    pub val:u64,
}

enum PageAttr {
    Present = 0,
    Writable = 1,
    UserAsc = 2,
    WriteTroughCache = 3,
    DisbleCache = 4,
    Accessed = 5,
    Dirty = 6,
    HugePage = 7,
    Global = 8,
    NoExec = 63,
}

impl PageTableEntry {
    ///attr 0: present bit
    ///attr 1: writable bit
    ///attr 2: user accessible bit
    ///attr 3: write through cache bit
    ///attr 4: disable cache bit
    ///attr 5: accessed bit
    ///attr 6: dirty bit
    ///attr 7: huge page bit
    ///attr 8: global bit
    ///attr 9-11:OS free use
    ///attr 12-51: physic address
    ///attr 52-62:OS free use
    ///attr 63: No exec bit
    pub fn set_config(&mut self, v:bool, attr:u8) {
        if v {
            self.val |= (v as u64) << attr;
        } else {
            self.val &= (v as u64) << attr;
        }
    }

    pub fn set_phyc_addr(&mut self, addr:u64) {
        for i in 0..40 {
            let b = addr << (64 - i - 1) >> (64 - i - 1);

            if b == 1 {
                self.val |= b << (52 + i);
            } else {
                self.val &= b << (52 + i);
            }
        }
    }
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}