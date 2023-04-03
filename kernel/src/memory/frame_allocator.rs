use super::allocator::LFRAME_ALLOCATOR;

use crate::config::{KERNEL_HEAP_SIZE, PAGE_SIZE,PHYS_MEM_END,INIT_PHYS_VIRT_OFFSET};
pub struct GFrameAllocator{}

lazy_static!{
    pub static ref FRAME_ALLOCATOR:GFrameAllocator=GFrameAllocator::new();
}

impl GFrameAllocator {
    pub fn new()->GFrameAllocator {
        Self {  }
    }
    pub fn init(&mut self){
        extern "C" {
            fn ekernel();
        }
        let frame_start=((ekernel as usize)+PAGE_SIZE-1)/PAGE_SIZE;
        let frame_end=(PHYS_MEM_END+INIT_PHYS_VIRT_OFFSET)/PAGE_SIZE;
        unsafe{
            LFRAME_ALLOCATOR
                .lock()
                .add_frame(usize::from(frame_start),usize::from(frame_end)
            );
        }
    }
    
}



