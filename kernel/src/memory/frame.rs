use riscv::register::ustatus;

use super::address::*;
use super::allocator::FRAME_ALLOCATOR;
use super::page_table::PTEFlags;

bitflags! {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq,Debug)]
    pub struct FrameFlags: u8 {
        const R = 1 << 1;//readable
        const W = 1 << 2;//writable
        const X = 1 << 3;//executable
        const U = 1 << 4;//user
    }
}




pub struct FrameArea{
    pub ppn:PhysPageNum,
    pub count:usize,
    pub flags:usize,
}

impl FrameArea {
    pub fn clear(&mut self){
        for i in usize::from(self.ppn)..usize::from(self.ppn)+self.count{
            let ppn=PhysPageNum::from(i);
            let bytes_array = ppn.get_bytes_array();
            for i in bytes_array {
                *i = 0;
            }
        }

    }
    pub fn new(ppn: PhysPageNum,count:usize,flags:FrameFlags) -> Self {
        // page cleaning
        let mut frame=FrameArea{ppn,count,flags:flags.bits() as usize};
        frame.clear();
        frame
    }
    pub fn new_without_clear(ppn: PhysPageNum,count:usize,flags:FrameFlags) -> Self {
        let frame=FrameArea{ppn,count,flags:flags.bits() as usize};
        frame
    }
    pub fn new_by_alloc(count:usize,flags:FrameFlags)->Option<Self>{
        let p=FRAME_ALLOCATOR.lock().alloc(count);
        match p {
            Some(ppn)=>Some(FrameArea::new(ppn,count,flags)),
            None => None,
        }
    }
    pub fn ppn(&self)->PhysPageNum{
        self.ppn
    }
    pub fn flags(&self)->FrameFlags{
        FrameFlags::from_bits(self.flags as u8).unwrap()
    }
    pub fn has_flags(&self,flags:FrameFlags)->bool{
         (self.flags()&flags)==flags
    }
    pub fn set_flags(&mut self,flags:FrameFlags){
        self.flags|=flags.bits() as usize;
    }
    pub fn unset_flags(&mut self,flags:FrameFlags){
        self.flags &= flags.bits() as usize;
    }
}

impl From<FrameFlags> for PTEFlags {
    fn from(value: FrameFlags) -> Self {
        PTEFlags::from_bits(value.bits()).unwrap()|PTEFlags::V
    }
}

impl Drop for FrameArea {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self.ppn,self.count);
    }
}
