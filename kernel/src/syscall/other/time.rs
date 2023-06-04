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

pub fn sys_gettimeofday(tv:*mut TimeVal,should_be_null:usize)->isize{
    if should_be_null!=0{
        return -1;
    }
    let vstart=VirtAddr::from(tv as usize);
    let vend=VirtAddr::from(tv as usize+size_of::<TimeVal>());
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::W)==false{
        return -1;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *mut TimeVal;
    unsafe{
        *vpt=TimeVal::get_timeval();
    } 
    0
}

pub fn sys_nanosleep(req:*const TimeSpec,rem:*const TimeSpec)->isize{
    let ts=TimeSpec::get_timespec();
    let vstart=VirtAddr::from(req as usize);
    let vend=VirtAddr::from(req as usize+size_of::<TimeSpec>());
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::R)==false{
        return -1;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *const TimeSpec;
    let req_ts=unsafe{
        *vpt
    }+ts;
    
    while TimeSpec::get_timespec()<req_ts{
        ;
    }
    0
}
