mod syscall_num;
mod other;
mod thread;
mod fs;
mod mm;
mod proc;
use core::mem::size_of;

use crate::config::USER_STRING_MAX_LEN;
use crate::fs::fs::fwrite;
use crate::memory::address::VirtAddr;
use crate::memory::address::VirtPageNum;
use crate::memory::page_table::PTEFlags;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::thread::*;
use crate::proc::scheduler::CURRENT_TID;
use crate::arch::cpu_id;
use crate::proc::uthread::TrapFrame;
use crate::timer::*;
use alloc::string::String;
use syscall_num::*;

use self::fs::{sys_getcwd, sys_chdir, sys_mkdirat, sys_openat, sys_close, sys_read, sys_write, sys_dup, sys_dup3};
use self::mm::sys_brk;
use self::other::*;
use self::other::time::*;
use self::proc::{sys_clone, sys_wait};
use thread::*;


pub fn syscall(syscall_id: usize, args: [usize; 6],tf:TrapFrame) -> isize{
    match syscall_id {
        SYS_WRITE => {
            sys_write(args[0] as isize,args[1] as *mut u8, args[2])
        },
        SYS_EXIT => {
            //println!("thread want to exit");
            sys_exit(args[0] as isize);
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
            sys_chdir(args[0] as *const u8)
        }
        SYS_MKDIRAT =>{
            sys_mkdirat(args[0] as i32,args[1] as *const u8,args[2])
        }
        SYS_OPENAT =>{
            sys_openat(args[0] as isize,args[1] as *const u8,args[2] as isize,args[3])
        }
        SYS_CLOSE =>{
            sys_close(args[0] as isize)
        }
        SYS_READ => {
            sys_read(args[0] as isize,args[1] as *mut u8,args[2])
        }
        SYS_DUP => {
            sys_dup(args[0] as isize)
        }
        SYS_DUP3=>{
            sys_dup3(args[0] as isize,args[1] as isize,args[2])
        }
        SYS_BRK=>{
            sys_brk(args[0])
        }
        SYS_CLONE=>{
            sys_clone(args[0], args[1], args[2], args[3], args[4],tf)
        }
        SYS_WAIT4=>{
            sys_wait(args[0] as isize,args[1],args[2])
        }
        SYS_SCHED_YIELD=>{
            yield_();
            0
        }
        SYS_GETPID=>{
            my_thread!().tid as isize
        }
        SYS_GETPPID=>{
            (match my_thread!().father_tid{
                None=>1,
                Some(tid)=>tid,
            }) as isize
        } 
        SYS_MOUNT=>{
            0
        }
        SYS_UMOUNT2=>{
            0
        }
        SYS_FADVISE64=>{
            0
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

pub fn get_user_string(s:*const u8)->Option<String>{
    let mut new_path=String::new();
    let mut vstart=VirtAddr::from(s as usize);
    let mut vpn=vstart.vpn();
    let mut vend=VirtAddr::from(VirtPageNum(vpn.0+1));
    loop  {
        if my_thread!().pgtable.check_user_range(vstart, vend,PTEFlags::R)==false{
            //println!("get_user_string: check_error");
            return None;
        }
        let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *const u8;
        let len=vend.0-vstart.0;
        for i in 0..len{
            unsafe{
                if *vpt.add(i)==0{
                    return Some(new_path);
                }
                new_path.push(char::from_u32((*vpt.add(i)) as u32).unwrap());
            }
            if new_path.len()>USER_STRING_MAX_LEN{
                //println!("get_user_string: user_string too long");
                return None;
            }
        }
        vpn.0+=1;
        vstart=VirtAddr::from(vpn);
        vend=VirtAddr::from(VirtPageNum(vpn.0+1));
    }
    panic!("get_user_string:error");
}
