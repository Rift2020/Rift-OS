use core::{cell::UnsafeCell, iter::empty};

use super::kthread::*;
use crate::{config::*, lang_items::TrustCell, memory::{page_table::PageTable, map_kernel}};
use alloc::{boxed::Box, vec::Vec};
use spin::*;

pub type Tid=usize;
pub type ExitCode=usize;
lazy_static!{
    pub static ref THREAD_POOL:TrustCell<ThreadPool> =TrustCell::new(ThreadPool::new(MAX_THREAD_NUM));
}

#[derive(Clone)]
pub enum Status {
    Ready,
    Running,
    LightSleeping,//可中断
    DeepSleeping,//不可中断
    Killed,
}

pub struct Thread{
    pub tid:Tid,
    pub pgtable:PageTable,
    pub kthread:Box<KThread>,
}

pub struct ThreadInfo{
    pub thread:Box<Thread>,
    status:Status,
}

pub struct ThreadPool{
    pub pool:Vec<Mutex< Option<ThreadInfo> >>,
}

impl Thread {
    pub fn empty()->Thread{
        Thread { tid: MAX_THREAD_NUM, pgtable:PageTable::empty() ,kthread: Box::new(KThread::empty())  }
    }
    pub fn new_thread_same_pgtable()->Thread{
        let pgtable=map_kernel();
        let root_ppn=pgtable.root_ppn;
        Thread{
            tid:THREAD_POOL.get_mut().alloc_tid(),
            pgtable,
            kthread:KThread::new_kthread(root_ppn),
        }
    }
}

impl ThreadPool {
    pub fn new(max_thread_num:usize)->ThreadPool {
        ThreadPool { 
            pool: {
                let mut vec=Vec::new();
                vec.resize_with(max_thread_num,Default::default);
                vec
            } 
        }
    }
    fn alloc_tid(&self)->Tid{
        for (i,thread) in self.pool.iter().enumerate(){
            if thread.lock().is_none(){
                return i;
            }
        }
        panic!("pool full ?");
    }
    pub fn insert(&mut self,thread:Box<Thread>){
        let tid=self.alloc_tid();
        *self.pool[tid].lock()=Some(
            ThreadInfo { thread, status: Status::Ready }
        );
        
    }
    pub fn remove(&mut self,tid:Tid){
        debug_assert!(self.pool[tid].lock().is_some());
        *self.pool[tid].lock()=None;
    }
    pub fn change_status(&mut self,tid:Tid,status:Status){
        debug_assert!(self.pool[tid].lock().is_some());
        self.pool[tid].lock().as_mut().unwrap().status=status;
    }
}
