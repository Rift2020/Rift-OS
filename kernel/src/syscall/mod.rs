use crate::memory::address::VirtAddr;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::thread::*;
use crate::proc::scheduler::CURRENT_TID;
use crate::arch::cpu_id;
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize{
    match syscall_id {
        SYS_WRITE => {
            
            let kva=usize::from(my_thread!().pgtable.user_va_to_kernel_va(VirtAddr::from(args[1]))) ;
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
            println!("thread want to exit");
            yield_();
            0
        },
        _ => {
            panic!("unknown syscall id {}", syscall_id);
        },
    }
}
