use crate::proc::kthread::*;
use crate::sbi::hart_start;
use crate::config::PHYS_KERNEL_START;

pub fn start_cpu_from_start2(cpu_id:usize)->KStack{
    extern "C" {
        fn _start();
        fn _start2();
    }
    let kstack=KStack::new(); 
    hart_start(cpu_id,PHYS_KERNEL_START+(_start2 as usize-_start as usize),kstack.top_addr());
    kstack
}


#[inline]
pub fn r_tp() -> usize {
    let tp;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) tp) };
    tp
}

#[inline]
pub fn cpu_id()->usize{
    //r_tp()
    0
}
