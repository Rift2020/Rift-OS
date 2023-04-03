#![allow(unused)]

pub const PAGE_SIZE:usize = 4096;
pub const KERNEL_HEAP_SIZE:usize = 128*1024;
//pub const PHYS_MEM_END:usize = 0xFDA4F580;//
pub const PHYS_MEM_START:usize=0x8000_0000;
pub const PHYS_MEM_SIZE:usize =0x8000_000;//128M
pub const PHYS_MEM_END:usize = PHYS_MEM_START+PHYS_MEM_SIZE;
pub const PHYS_VIRT_OFFSET:usize = 0xffff_ffc8_0000_0000-PHYS_MEM_START;
pub const FRAME_PHYS_VIRT_OFFSET:usize=PHYS_VIRT_OFFSET>>12;
pub const KERNEL_CODE_START_ADDR:usize = 0xffff_ffff_c000_0000;
pub const KERNEL_CODE_OFFSET:usize = KERNEL_CODE_START_ADDR - PHYS_MEM_START;
