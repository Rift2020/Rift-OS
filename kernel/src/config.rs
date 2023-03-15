#![allow(unused)]

pub const PAGE_SIZE:usize = 4096;
pub const KERNEL_HEAP_SIZE:usize = 128*1024;
//pub const PHYS_MEM_END:usize = 0xFDA4F580;//
pub const PHYS_MEM_SIZE:usize =0x8000000;//128M
pub const PHYS_MEM_END:usize = 0x88000000;
pub const INIT_PHYS_VIRT_OFFSET:usize = 0xffffffff40000000;
