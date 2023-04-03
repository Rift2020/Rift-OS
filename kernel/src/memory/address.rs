use core::fmt::Debug;
use bitflags::*;
//sv39
const PA_WIDTH: usize = 56;
const PAGE_OFFSET_WIDTH: usize = 12;
const PPN_WIDTH:usize = PA_WIDTH - PAGE_OFFSET_WIDTH;
const PAGE_SIZE: usize = 4096;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    fn from(v:usize)->Self{PhysAddr(v&(1<<PA_WIDTH)-1)}
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {PhysPageNum(v & ( (1 << PPN_WIDTH) - 1 )) }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
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
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
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
