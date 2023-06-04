pub mod allocator;
pub mod address;
pub mod page_table;
pub mod frame;
use crate::config::{INIT_KERNEL_HEAP_SIZE, PAGE_SIZE,PHYS_MEM_END,PHYS_VIRT_OFFSET,KERNEL_CODE_OFFSET,PHYS_MEM_START, PHYS_ACCESS_START, PHYS_MEM_SIZE, PHYS_SPACE_SIZE, KERNEL_CODE_START_ADDR};
#[global_allocator]
static HEAP_ALLOCATOR:allocator::LockedHeap::<32>  = allocator::LockedHeap::<32>::empty();
static mut HEAP_SPACE: [u8; INIT_KERNEL_HEAP_SIZE] = [0; INIT_KERNEL_HEAP_SIZE];
use allocator::FRAME_ALLOCATOR;
use address::*;
use frame::*;
use page_table::*;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
}

fn kernel_code_usize_to_ppn(v:usize)->PhysPageNum{
    assert_eq!((v-KERNEL_CODE_OFFSET)&(PAGE_SIZE-1),0);
    PhysPageNum((v-KERNEL_CODE_OFFSET)>>PAGE_OFFSET_WIDTH)
}

pub fn map_kernel()->PageTable{
    assert_eq!(ebss as usize,ekernel as usize);
    let stext_vpn=VirtPageNum::from(VirtAddr::from(stext as usize));
    let etext_vpn=VirtPageNum::from(VirtAddr::from(etext as usize));
    let srodata_vpn=VirtPageNum::from(VirtAddr::from(srodata as usize));
    let erodata_vpn=VirtPageNum::from(VirtAddr::from(erodata as usize));
    let sdata_vpn=VirtPageNum::from(VirtAddr::from(sdata as usize));
    let edata_vpn=VirtPageNum::from(VirtAddr::from(edata as usize));
    let sbss_vpn=VirtPageNum::from(VirtAddr::from(sbss_with_stack as usize));
    let ebss_vpn=VirtPageNum::from(VirtAddr::from(ebss as usize));
    
    let stext_ppn=kernel_code_usize_to_ppn(stext as usize);
    let etext_ppn=kernel_code_usize_to_ppn(etext as usize);
    let srodata_ppn=kernel_code_usize_to_ppn(srodata as usize);
    let erodata_ppn=kernel_code_usize_to_ppn(erodata as usize);
    let sdata_ppn=kernel_code_usize_to_ppn(sdata as usize);
    let edata_ppn=kernel_code_usize_to_ppn(edata as usize);
    let sbss_ppn=kernel_code_usize_to_ppn(sbss_with_stack as usize);
    let ebss_ppn=kernel_code_usize_to_ppn(ebss as usize);
    
    let text_frame=FrameArea::new_without_clear(stext_ppn,stext_ppn.page_count(etext_ppn),FrameFlags::R|FrameFlags::X|FrameFlags::N);
    let rodata_frame=FrameArea::new_without_clear(srodata_ppn,srodata_ppn.page_count(erodata_ppn),FrameFlags::R|FrameFlags::N);
    let data_frame=FrameArea::new_without_clear(sdata_ppn,sdata_ppn.page_count(edata_ppn),FrameFlags::R|FrameFlags::W|FrameFlags::N);
    let bss_frame=FrameArea::new_without_clear(sbss_ppn,sbss_ppn.page_count(ebss_ppn),FrameFlags::R|FrameFlags::W|FrameFlags::N);

    let phys_mem_access_frame=FrameArea::new_without_clear(PhysPageNum::from(0),(PHYS_SPACE_SIZE+PAGE_SIZE-1)/PAGE_SIZE,FrameFlags::R|FrameFlags::W|FrameFlags::N);

    let mut pgtable=PageTable::new();
    pgtable.map(stext_vpn,text_frame);
    pgtable.map(srodata_vpn,rodata_frame);
    pgtable.map(sdata_vpn,data_frame);
    pgtable.map(sbss_vpn,bss_frame);
    pgtable.map(VirtPageNum::from(VirtAddr::from(PHYS_ACCESS_START)),phys_mem_access_frame);
    pgtable
}

pub fn init()->PageTable{
    extern "C" {
        fn ekernel();
    }
    println!("[Rift os] init_memory!");
    let frame_start=((ekernel as usize)-KERNEL_CODE_OFFSET+PHYS_VIRT_OFFSET+PAGE_SIZE-1)/PAGE_SIZE;
    let frame_end=(PHYS_MEM_END+PHYS_VIRT_OFFSET)/PAGE_SIZE;
    println!("fs,fe {:#x} {:#x}",frame_start,frame_end);
    unsafe{
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, INIT_KERNEL_HEAP_SIZE);
        FRAME_ALLOCATOR
            .lock()
            .add_frame(usize::from(frame_start),usize::from(frame_end));
    }
    map_kernel()
}

pub fn test(){
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    println!("heap test start");
    //检查初始堆空间能否分配
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..5 {
        v.push(i);
    }
    for i in 0..5 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    //检查堆空间能否动态增加
    v.clear();
    for i in 0..INIT_KERNEL_HEAP_SIZE*2 {
        v.push(i);
    }
    for i in 0..INIT_KERNEL_HEAP_SIZE*2 {
        assert_eq!(v[i], i);
    }
    assert!(!bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
