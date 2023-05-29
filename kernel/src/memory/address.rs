use core::{fmt::Debug, ops, debug_assert};
use bitflags::*;

use crate::config::{PHYS_VIRT_OFFSET, FRAME_PHYS_VIRT_OFFSET,PAGE_SIZE};
//sv39
pub const PA_WIDTH: usize = 56;
pub const VA_WIDTH: usize =39;
pub const PAGE_OFFSET_WIDTH: usize = 12;
pub const PPN_WIDTH:usize = PA_WIDTH - PAGE_OFFSET_WIDTH;
pub const VPN_WIDTH:usize = VA_WIDTH - PAGE_OFFSET_WIDTH;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    fn from(v:usize)->Self{
        debug_assert!(v&(1<<PA_WIDTH)-1==v);
        PhysAddr(v&(1<<PA_WIDTH)-1)
    }
}

impl From<usize> for VirtAddr{
    fn from(value: usize) -> Self {
        let one_26:usize=(1<<(64-38))-1;
        assert!((value>>38)==0||((value>>38)&one_26)==one_26);
        Self(value&((1<<39)-1))
    }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        PhysPageNum(v & ( (1 << PPN_WIDTH) - 1 )) 
    }
}

impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        VirtPageNum(value&((1<<VPN_WIDTH)-1))
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
}

impl From<VirtPageNum> for usize {
    fn from(value: VirtPageNum) -> Self {
        value.0
    }
}

impl PhysAddr {
    pub fn page_offset(&self)->usize{self.0 & (PAGE_SIZE-1)}
    pub fn floor_ppn(&self) -> PhysPageNum { PhysPageNum(self.0 / PAGE_SIZE) }
    pub fn ceil_ppn(&self) -> PhysPageNum { PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE) }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(),0);
        v.floor_ppn()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        PhysAddr(v.0<<PAGE_OFFSET_WIDTH)
    }
}
impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        let va=pa_to_va_usize(pa);
        unsafe { core::slice::from_raw_parts_mut(va as *mut u8, 4096) }
    }
    pub fn set_bytes_array(&mut self,src:*const u8){
        let pa: PhysAddr = (*self).into();
        let va=pa_to_va_usize(pa);
        let ptr=va as *mut u8;
        unsafe {
            core::ptr::copy(src,ptr,4096);
        }
    }
    pub fn page_count(&self,rhs:PhysPageNum)->usize{
        rhs.0-self.0
    }
    pub fn satp(&self)->usize{
        ((riscv::register::satp::read().bits()>>PPN_WIDTH)<<PPN_WIDTH)|self.0
    }

}

impl VirtAddr {
    pub fn floor_vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil_vpn(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    pub fn vpn(&self)->VirtPageNum{
        self.floor_vpn()
    }
    pub fn offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.offset() == 0
    }
}

impl VirtPageNum {
    pub fn indexes(&self)->[usize;3]{
        let vpn=self.0;
        [vpn&((1<<9)-1),(vpn>>9)&((1<<9)-1),(vpn>>18)&((1<<9)-1)]
    }
}

impl From<VirtAddr> for VirtPageNum{
    fn from(value: VirtAddr) -> Self {
        assert!(value.aligned());
        VirtPageNum(value.0>>PAGE_OFFSET_WIDTH)
    }
}

impl ops::Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;
    fn add(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr(self.0+rhs.0)
    }
}

impl ops::Add<VirtAddr> for VirtAddr {
    type Output = VirtAddr;
    fn add(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr(self.0+rhs.0)
    }
}



impl Debug for PhysAddr{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}",self.0))
    } 
}
impl Debug for VirtAddr{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}",self.0))
    }
}
impl Debug for PhysPageNum{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}",self.0))
    }
}
impl Debug for VirtPageNum{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}",self.0))
    }
}


pub fn pa_to_va_usize(pa:PhysAddr)->usize{
    usize::from(pa)+PHYS_VIRT_OFFSET 
}
pub fn pa_usize_to_va_usize(pa:usize)->usize{
    pa+PHYS_VIRT_OFFSET
}
pub fn va_usize_to_pa_usize(va:usize)->usize{
    va-PHYS_VIRT_OFFSET
}
pub fn va_usize_to_pa(va:usize)->PhysAddr{
    PhysAddr::from(va-PHYS_VIRT_OFFSET)
}
