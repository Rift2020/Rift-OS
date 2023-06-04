use crate::timer::*;
use core::mem::size_of;
use crate::memory::address::VirtAddr;

use crate::arch::cpu_id;
use crate::my_thread;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::*;
use crate::memory::page_table::*;



pub fn sys_times(tms:*mut Tms)->isize{
    let vstart=VirtAddr::from(tms as usize);
    let vend=VirtAddr::from(tms as usize+size_of::<Tms>());
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::W)==false{
        return -1;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *mut Tms;
    unsafe{
        *vpt=my_thread!().tms;
    }
    0
}
