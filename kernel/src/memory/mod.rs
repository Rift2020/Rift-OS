mod allocator;
mod address;
use crate::config::{KERNEL_HEAP_SIZE, PAGE_SIZE,PHYS_MEM_END,INIT_PHYS_VIRT_OFFSET};
#[global_allocator]
static HEAP_ALLOCATOR:allocator::LockedHeap::<32>  = allocator::LockedHeap::<32>::empty();
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
use allocator::FRAME_ALLOCATOR;
use address::*;
pub fn init(){
    extern "C" {
        fn ekernel();
    }
    let frame_start=((ekernel as usize)+PAGE_SIZE-1)/PAGE_SIZE;
    let frame_end=(PHYS_MEM_END+INIT_PHYS_VIRT_OFFSET)/PAGE_SIZE;
    unsafe{
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
        FRAME_ALLOCATOR
            .lock()
            .add_frame(usize::from(frame_start),usize::from(frame_end));
    }
}

pub fn test(){
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
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
    for i in 0..KERNEL_HEAP_SIZE*2 {
        v.push(i);
    }
    for i in 0..KERNEL_HEAP_SIZE*2 {
        assert_eq!(v[i], i);
    }
    assert!(!bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
