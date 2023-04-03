use core::fmt::Debug;

use bitflags::*;
use super::address::*;

bitflags! {
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq,Debug)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0;//valid
        const R = 1 << 1;//readable
        const W = 1 << 2;//writable
        const X = 1 << 3;//executable
        const U = 1 << 4;//user
        const G = 1 << 5;//global
        const A = 1 << 6;//accessed
        const D = 1 << 7;//dirty
        //RSW 8,9
    }
}


#[derive(Copy, Clone)]
#[repr(C)]
//[63:54]Reserved,[53:10]PPN,[9:0]flags
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry {
            bits: 0,
        }
    } 
 
    pub fn get_ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    } 
    pub fn set_ppn(&mut self,ppn: PhysPageNum){
        debug_assert!(usize::from(ppn)<(1<<44));
        self.bits = (usize::from(ppn)<<10)+(self.bits&(0xffff_ffff_ffff_ffff<<10));
        debug_assert!(ppn==self.get_ppn());
    }   

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn is_directory(&self)->bool{
        let flag=(PTEFlags::R|PTEFlags::W|PTEFlags::X|PTEFlags::U);
        self.is_valid() && ((self.flags()&flag)==PTEFlags::empty())
    }
    pub fn is_leaf(&self) -> bool {
        let mask = PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::U;
        self.is_valid() && ((self.flags() & mask) != PTEFlags::empty())
    }
    pub fn has_flags(&self,flag:PTEFlags)->bool{
        (self.flags()&flag)==flag
    }
    pub fn set_flags(&mut self,flag:PTEFlags){
        self.bits |= flag.bits() as usize;
    }
    pub fn unset_flags(&mut self,flag:PTEFlags){
        self.bits &= flag.bits() as usize;
    }
}

impl Debug for PageTableEntry{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PTE:\n\t{:?}\n\t{:#b}",self.get_ppn(),self.flags().bits()))
    }
}
