use core::mem::forget;

use riscv::register::ustatus;

use crate::config::PAGE_SIZE;

use super::address::*;
use super::allocator::FRAME_ALLOCATOR;
use super::page_table::PTEFlags;

use xmas_elf::program::Flags;

bitflags! {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq,Debug)]
    pub struct FrameFlags: u8 {
        const R = 1 << 1;//readable
        const W = 1 << 2;//writable
        const X = 1 << 3;//executable
        const U = 1 << 4;//user
        const N = 1 << 5;//do not dealloc ! 
    }
}




pub struct FrameArea{
    pub ppn:PhysPageNum,
    pub count:usize,
    pub flags:usize,
}

impl FrameArea {
    pub fn clear_pages(ppn: PhysPageNum,count:usize){
        for i in usize::from(ppn)..usize::from(ppn)+count{
            let ppn=PhysPageNum::from(i);
            let bytes_array = ppn.get_bytes_array();
            for i in bytes_array {
                *i = 0;
            }
        }
    }
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
    pub fn new_by_copy_slice(v_ptr:*const u8,byte_len:usize,flags:FrameFlags)->Option<Self>{ 
        assert!(byte_len!=0);
        let count=(byte_len+PAGE_SIZE-1)/PAGE_SIZE;
        let p=FRAME_ALLOCATOR.lock().alloc(count);
        match p {
            Some(ppn)=>{
                let v_p=pa_to_va_usize(PhysAddr::from(ppn)) as *mut u8;
                unsafe{
                    core::ptr::copy(v_ptr,v_p,byte_len);
                }
                Some(FrameArea::new_without_clear(ppn,count,flags))
            },
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
        let ret=PTEFlags::from_bits((value&(!FrameFlags::N)).bits()).unwrap()|PTEFlags::V;
        ret
    }
}

impl From<Flags> for FrameFlags {
    fn from(value: Flags) -> Self {
        let mut frameflag=FrameFlags::empty();
        if value.is_read(){
            frameflag|=FrameFlags::R;
        }
        if value.is_write(){
            frameflag|=FrameFlags::W;
        }
        if value.is_execute(){
            frameflag|=FrameFlags::X;
        }
        frameflag|FrameFlags::U
    }
}

impl Drop for FrameArea {
    fn drop(&mut self) {
        if self.has_flags(FrameFlags::N)==false{
            FRAME_ALLOCATOR.lock().dealloc(self.ppn,self.count);
        }
    }
}
