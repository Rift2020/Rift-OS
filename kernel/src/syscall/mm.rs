use core::clone;
use core::mem::size_of;
use core::ptr::slice_from_raw_parts;
use core::ptr::slice_from_raw_parts_mut;
use crate::memory::address::*;

use crate::arch::cpu_id;
use crate::my_lock;
use crate::my_thread;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::*;
use crate::memory::page_table::*;

use super::get_user_string;
use super::user_buf_to_vptr;

pub fn sys_brk(_brk:usize)->isize{
    let mut lk=my_lock!();
    let _brk=_brk>>7;
    let brk_value=lk.as_ref().unwrap().thread.pgtable.get_brk()+VirtAddr::from(_brk);
    if _brk==0{
        return brk_value.0 as isize;
    }
    lk.as_mut().unwrap().thread.pgtable.set_brk(brk_value);
    0
}
