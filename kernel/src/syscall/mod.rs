mod syscall_num;
mod other;
use core::mem::size_of;

use crate::memory::address::VirtAddr;
use crate::memory::page_table::PTEFlags;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::thread::*;
use crate::proc::scheduler::CURRENT_TID;
use crate::arch::cpu_id;
use syscall_num::*;

use self::other::*;


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
            //TODO
            println!("thread want to exit");
            yield_();
            0
        },
        SYS_UNAME => {
            sys_uname(args[0] as *const Utsname)
        }
        _ => {
            panic!("unknown syscall id {}", syscall_id);
        },
    }
}


