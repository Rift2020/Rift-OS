mod syscall_num;
mod other;
mod thread;
mod fs;
use core::mem::size_of;

use crate::memory::address::VirtAddr;
use crate::memory::page_table::PTEFlags;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::thread::*;
use crate::proc::scheduler::CURRENT_TID;
use crate::arch::cpu_id;
use crate::timer::*;
use syscall_num::*;

use self::fs::{sys_getcwd, sys_chdir, sys_mkdirat};
use self::other::*;
use self::other::time::*;
use thread::*;


pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize{
    match syscall_id {
        SYS_WRITE => {
            let user_buf=VirtAddr::from(args[1]);
            let user_buf_end=VirtAddr::from(args[1]+args[2]*size_of::<u8>());
            if my_thread!().pgtable.check_user_range(user_buf,user_buf_end,PTEFlags::R)==false{
                return -1;
            }
            let kva=my_thread!().pgtable.uva_to_kusize(VirtAddr::from(args[1])) ;
            let buf:*const u8=kva as *const u8;
            unsafe{
                let count=args[2];
                for i in 0..count{
                    print!("{}",*(buf.add(i)) as char);
                }
            }
            args[2] as isize
        },
        SYS_EXIT => {
            //println!("thread want to exit");
            sys_exit();
            panic!("exit fail");
            -1
        },
        SYS_UNAME => {
            sys_uname(args[0] as *mut Utsname)
        }
        SYS_TIMES => {
            sys_times(args[0] as *mut Tms)
        }
        SYS_GETTIMEOFDAY => {
            sys_gettimeofday(args[0] as *mut TimeVal,args[1])
        }
        SYS_NANOSLEEP =>{
            sys_nanosleep(args[0] as *const TimeSpec,args[1] as *mut TimeSpec)
        }
        SYS_GETCWD =>{
            sys_getcwd(args[0] as *mut u8,args[1])
        }
        SYS_CHDIR =>{
            sys_chdir(args[0] as *const char)
        }
        SYS_MKDIRAT =>{
            sys_mkdirat(args[0] as i32,args[1] as *const char,args[2])
        }


        _ => {
            panic!("unknown syscall id {}", syscall_id);
        },
    }
}

pub fn user_buf_to_vptr(buf:usize,byte_len:usize,flag:PTEFlags)->Option<usize>{
    let vstart=VirtAddr::from(buf as usize);
    let vend=VirtAddr::from(buf as usize+byte_len);
    if my_thread!().pgtable.check_user_range(vstart,vend,flag)==false{
        return None;
    }
    Some(my_thread!().pgtable.uva_to_kusize(vstart))


}


