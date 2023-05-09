use alloc::boxed::Box;

use super::thread::*;
use super::scheduler::CURRENT_TID;
use super::scheduler::IDLE_TID;
use crate::arch::cpu_id;
use crate::config::*;
use crate::memory::address::*;
use crate::memory::allocator::FRAME_ALLOCATOR;
use core::arch::asm;
use riscv;


pub static mut INIT_KTHREAD:KThread=KThread::empty();
pub static mut CURRENT_KTHREAD:[Option<*mut KThread>;CPU_NUM]=[None;2];

#[repr(C)]
#[derive(Debug)]
pub struct Context{
    pub ra: usize,
    root_ppn: usize,
    s: [usize; 12],
}


pub struct KStack{
    va:usize,
}


pub struct KThread{
    pub context_addr:usize,
    pub kstack: KStack,
}

impl KStack {
    pub const fn empty()->Self{
        KStack { va: 0 }
    }
    pub fn new()->Self{
        KStack { va: pa_to_usize(PhysAddr::from(FRAME_ALLOCATOR.lock().alloc(KSTACK_PAGE_COUNT).unwrap()))  }
    }
    pub fn top_addr(&self)->usize{
        self.va+PAGE_SIZE*KSTACK_PAGE_COUNT
    }
}
impl Drop for KStack{
    fn drop(&mut self) {
        if self.va!=0{
            FRAME_ALLOCATOR.lock().dealloc(PhysPageNum::from(usize_to_pa(self.va)),KSTACK_PAGE_COUNT);
        }
    }
}

#[naked]
pub unsafe extern "C" fn switch(current_context_va: &mut usize, target_context_va:&mut usize) {
    asm!(include_str!("switch.asm"),options(noreturn));
}
//出让cpu，yield是保留字，你不应该在持有着任何锁时调用该函数
pub fn yield_(){
    let current_tid=CURRENT_TID.lock()[cpu_id()];//这一行右边表达式不能直接移到下一行current_tid处，会造成死锁(switch_to也要CURRENT_TID的锁)
    THREAD_POOL.get_mut().pool[current_tid].lock().as_mut().unwrap().thread.kthread.switch_to_idle();
}

#[repr(align(4))]
pub fn forkret(){
    println!("hi! switch success! this is a new thread");
    //let current_tid:usize={CURRENT_TID.lock()[cpu_id()]};
    //如果一个线程曾经切换出去过，那么再次回来后，他将会在switch_to最末尾隐式地释放掉自己进程和目标进程(一般为idle)的锁
    //但是新初始的进程不会这么做，所以我们必须显式地释放他们
    unsafe{
        THREAD_POOL.get_mut().pool[CURRENT_TID.lock()[cpu_id()]].force_unlock();
        THREAD_POOL.get_mut().pool[IDLE_TID.lock()[cpu_id()]].force_unlock();
    }
    loop {
        println!("hi,i'm cutest thread {} ,hi~, now switch to idle_thread !!",CURRENT_TID.lock()[cpu_id()]);
        yield_();
    }
        
}

impl Context {
    pub const fn empty()->Context{
        Context { ra: 0, root_ppn: 0, s: [0;12] }
    }
    pub fn new_context(satp:usize)->Context{
        Context {
            ra: (forkret as usize),
            root_ppn: (satp),
            s: ([0;12]) 
        }
    }
    unsafe fn push_at(self,stack_top_addr:usize)->usize{
        let ptr=(stack_top_addr as *mut Context).sub(1);
        *ptr=self;
        ptr as usize
    }
}    

impl KThread {
    pub fn switch_to(&mut self,target_tid:Tid){
        CURRENT_TID.lock()[cpu_id()]=target_tid;
        unsafe{
            let idle_thread=&mut (THREAD_POOL.get_mut().pool[target_tid].lock());
            switch(&mut self.context_addr,&mut (idle_thread.as_mut().unwrap().thread.kthread.context_addr));

        }

    }
    pub fn switch_to_idle(&mut self){
        let idle_tid={IDLE_TID.lock()[cpu_id()]};
        unsafe{ IDLE_TID.force_unlock();
        };
        self.switch_to(idle_tid);
    }
    pub const fn empty()->KThread{
        KThread { context_addr: 0, kstack: (KStack::empty()) }
    }
    pub fn new_kthread_same_pgtable()->Box<KThread>{
        let kstack=KStack::new();
        let context=Context::new_context(riscv::register::satp::read().bits());
        let context_addr=unsafe{
            context.push_at(kstack.top_addr())
        };
        Box::new(KThread {context_addr,kstack})
    }
}
