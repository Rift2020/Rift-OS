use alloc::string::String;

use crate::fs::fs::*;
use crate::fs::FILE_SYSTEM;
use core::mem::size_of;
use crate::memory::address::VirtAddr;

use crate::arch::cpu_id;
use crate::my_thread;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::*;
use crate::memory::page_table::*;
pub fn sys_getcwd(buf:*mut char,size:usize)->isize{
    if buf as usize ==0 {
        println!("not impl for now");
        return 0;
    }
    let vstart=VirtAddr::from(buf as usize);
    let vend=VirtAddr::from(buf as usize+size_of::<char>()*size);
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::W)==false{
        return 0;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *mut u8;
    let cwd=String::from("/")+my_thread!().cwd.as_str();
    if cwd.len()>size{
        return 0;
    }
    unsafe{
        let mut vslice=core::slice::from_raw_parts_mut(vpt,size);
        vslice[..cwd.len()].copy_from_slice(cwd.as_bytes());
    }
    buf as isize
}
