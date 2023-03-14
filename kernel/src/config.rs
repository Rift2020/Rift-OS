#![allow(unused)]

pub const PAGE_SIZE:usize = 4096;
pub const KERNEL_HEAP_SIZE:usize = 128*1024;
pub const PHYS_MEM_END:usize = 0xFDA4F580;
pub const PHYS_VIRT_OFFSET:usize = 0xffffffff40000000;
