use core::{fmt::Debug, iter::empty, mem::size_of};
use alloc::vec::{self, Vec};
use crate::config::PAGE_SIZE;
use bitflags::*;
use riscv::asm::sfence_vma_all;
use super::{address::*, frame::{FrameArea, self, FrameFlags}, allocator::FRAME_ALLOCATOR};

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
        //RSW 8,9 使用RSW位可能要更改u8
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
    pub unsafe fn get_pte(ppn:PhysPageNum,index:usize)->&'static mut PageTableEntry{
        (pa_to_va_usize(PhysAddr::from(ppn)+PhysAddr::from(size_of::<usize>()*index)) as *mut PageTableEntry).as_mut().unwrap()
    }
}

impl Debug for PageTableEntry{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PTE: {:?}\tflags: {:#b}",self.get_ppn(),self.flags().bits()))
    }
}

pub struct PageTable{
    pub root_ppn:PhysPageNum,
    pub frame_set:Vec<FrameArea>,
}



impl PageTable {
    pub const fn empty()->Self{
        PageTable { root_ppn: PhysPageNum(0), frame_set: Vec::new() }
    }
    pub fn new()->Self {
        let frame=FrameArea::new_by_alloc(1,FrameFlags::R|FrameFlags::W).unwrap();
        let mut pgtable=Self{root_ppn:frame.ppn,frame_set:Vec::new()};
        pgtable.frame_set.push(frame);
        pgtable
    }
    //仅可作为临时使用
    pub fn new_by_ppn(root_ppn:PhysPageNum)->Self{
        Self { root_ppn: (root_ppn), frame_set: Vec::new() }
    }

    //depth必须为[0,2]
    fn print_pgtable(root_ppn:PhysPageNum,depth:u8){
        assert!(depth<=2);
        if depth==0{
            println!("PageTable: {:?} :",root_ppn);
        }
        let ppn=root_ppn;
        for i in 0..512{
            unsafe{
                let pte=PageTableEntry::get_pte(ppn, i);
                if pte.is_valid()==false{
                    continue;
                }
                for j in  0..depth{
                    print!("\t");
                }
                println!("{:>3} {:?}",i,pte);
                if pte.is_directory() {
                    Self::print_pgtable(pte.get_ppn(),depth+1);
                }
            }
        }

    }
    pub fn alloc_pte(&mut self,pte:&mut PageTableEntry)->bool{
        let frame=FrameArea::new_by_alloc(1,FrameFlags::R|FrameFlags::W);
        match frame {
            Some(f)=>{
                pte.set_ppn(f.ppn);
                pte.set_flags(PTEFlags::V);
                self.frame_set.push(f);
                true
            }
            None=>{
                false
            }
        }
    }

    pub fn find_va_pa(&mut self,va:VirtAddr)->PhysAddr{
        let pte=self.find_pte(va.vpn(),false).unwrap();
        let ppn=pte.get_ppn();
        let offset=va.offset();
        let pa=PhysAddr::from(ppn);
        PhysAddr(usize::from(pa)+offset)
    }
    pub fn user_va_to_kernel_va(&mut self,va:VirtAddr)->VirtAddr{
        let pa=self.find_va_pa(va);
        pa_to_va(pa)
    }
    pub fn find_pte(&mut self,vpn:VirtPageNum,alloc:bool)->Option<&mut PageTableEntry>{
        let vpn=vpn.indexes();
        let mut ppn=self.root_ppn;
        for i in  (0..3).rev(){
            let pte=unsafe {PageTableEntry::get_pte(ppn,vpn[i])};
            if  i!=0&&pte.is_valid()==false{
                if alloc==false{
                    return None;
                }
                self.alloc_pte(pte);
            }
            if i==0{
                return Some(pte);
            }
            ppn=pte.get_ppn();
        }
        panic!("find_pte error");
        
    }
    pub fn map(&mut self,vpn:VirtPageNum,frame:FrameArea)->bool{
        for i in 0..frame.count{
            let v=usize::from(vpn)+i;
            let p=usize::from(frame.ppn)+i;
            let pte=self.find_pte(VirtPageNum::from(v),true);
            match pte {
                Some(pte1)=>{
                    pte1.set_ppn(PhysPageNum::from(p));
                    pte1.set_flags(PTEFlags::from(frame.flags()));
                }
                None=>{
                    panic!("map fail!");
                    return false;
                }
            }
        }
        self.frame_set.push(frame);
        true
    }

    pub fn alloc_and_map(&mut self,vpn:VirtPageNum,count:usize,flags:FrameFlags)->bool{
        let frame=FrameArea::new_by_alloc(count, flags).unwrap();
        self.map(vpn, frame)
    }

    pub fn size_to_pgcount(size:usize)->usize{
        (size+PAGE_SIZE-1)/PAGE_SIZE
    }

    pub fn print(&self){
        Self::print_pgtable(self.root_ppn,0);
    }

    pub fn set_satp_to_root(&self){
        let satp=self.root_ppn.satp();
        riscv::register::satp::write(satp); 
        unsafe {sfence_vma_all();}
    }
}

impl Clone for PageTable{
    fn clone(&self) -> Self {
        let mut new_pgtable=Self::new();
        new_pgtable.root_ppn.set_bytes_array(self.root_ppn.get_bytes_array().as_ptr());
        for i in 1..self.frame_set.len(){
            //TODO!!
            //不是所有的frameArea都可以简单的搬起来
        }
        new_pgtable
    }
}

impl Debug for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.print();
        f.write_fmt(format_args!("")) 
    }
}
