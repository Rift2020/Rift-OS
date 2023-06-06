use crate::memory::{page_table::{PageTable}, frame::{FrameArea, FrameFlags}};
use crate::memory::address::*;
use xmas_elf::{
    header,
    program::{ Flags, SegmentData, Type },
    ElfFile,
};

pub trait ElfExt {
    fn entry_addr(&self)->usize;
    fn add_memory_area(&self,pgtable:&mut PageTable);
}

impl ElfExt for ElfFile<'_> {
    fn entry_addr(&self)->usize {
        assert!(self.header.pt2.type_().as_type()==header::Type::Executable);
        self.header.pt2.entry_point() as usize
    }
   fn add_memory_area(&self,pgtable:&mut PageTable) {
       for ph in self.program_iter(){
           if ph.get_type()!=Ok(Type::Load){
               continue;
           }
           let va=VirtAddr::from(ph.virtual_addr() as usize);
           let mem_size =ph.mem_size() as usize;
           pgtable.set_brk(VirtAddr(va.0+mem_size));
            let data = match ph.get_data(self).unwrap() {
                SegmentData::Undefined(data) => data,
                _ => panic!(),
            };
            let frame=FrameArea::new_by_copy_slice(data.as_ptr(),mem_size,FrameFlags::U|(ph.flags().into())).unwrap();
            pgtable.map(VirtPageNum::from(va),frame);
       }
   } 
}
