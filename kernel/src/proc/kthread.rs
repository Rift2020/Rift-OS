use alloc::boxed::Box;

use crate::config::*;
use crate::memory::address::*;
use crate::memory::allocator::FRAME_ALLOCATOR;
use core::arch::asm;
use riscv;
/*
lazy_static!{
    pub static ref INIT_KTHREAD:KThread=KThread::empty();
}
*/

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
        if(self.va!=0){
            FRAME_ALLOCATOR.lock().dealloc(PhysPageNum::from(usize_to_pa(self.va)),KSTACK_PAGE_COUNT);
        }
    }
}

    #[naked]
    pub unsafe extern "C" fn switch(current_context_va: &mut usize, target_context_va:&mut usize) {
        asm!(include_str!("switch.asm"),options(noreturn));
    }



#[repr(align(4))]
pub fn forkret(){
    println!("hi! switch success! this is thread2");
    println!("now switch to init_thread !!");
    unsafe {
        let mut current_context_addr=(*CURRENT_KTHREAD[0].unwrap()).context_addr;
        switch(&mut current_context_addr,&mut INIT_KTHREAD.context_addr);   
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
    pub fn switch_to(&mut self,target:&mut KThread){
        unsafe{
            CURRENT_KTHREAD[0]=Some(target as *mut KThread);
            switch(&mut self.context_addr,&mut target.context_addr);
        }
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
