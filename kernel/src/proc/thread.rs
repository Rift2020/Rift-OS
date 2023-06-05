use core::{cell::UnsafeCell, iter::empty};

use super::{kthread::*, uthread::UThread, scheduler::CURRENT_TID};
use crate::{config::*, lang_items::TrustCell, memory::{page_table::PageTable, map_kernel}, arch::cpu_id, timer::Tms, fs::fs::{File_, FileInner}};
use alloc::{boxed::Box, vec::Vec, string::String};
use spin::*;
use xmas_elf::ElfFile;
use xmas_elf::header;
use crate::proc::elf::*;

pub type Tid=usize;
pub type ExitCode=usize;
lazy_static!{
    pub static ref THREAD_POOL:TrustCell<ThreadPool> =TrustCell::new(ThreadPool::new(MAX_THREAD_NUM));
}

#[derive(Clone,PartialEq, Eq)]
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
    pub tms:Tms,
    pub cwd:String,
    pub fd_table:Vec<Option<FileInner>>,
    pub kthread:Box<KThread>,
    pub uthread:Box<UThread>,
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
        Thread { tid: MAX_THREAD_NUM, pgtable:PageTable::empty() ,tms:Tms::empty(),cwd:String::new(),fd_table:Vec::new(),kthread: Box::new(KThread::empty()),uthread:Box::new(UThread::empty()) }
    }
    pub fn new_thread_same_pgtable()->Thread{
        let pgtable=map_kernel();
        let root_ppn=pgtable.root_ppn;
        Thread{
            tid:MAX_THREAD_NUM,
            pgtable,
            tms:Tms::empty(),
            cwd:String::new(),
            kthread:KThread::new_kthread(root_ppn),
            fd_table:{
                let mut v=Vec::new();
                v.push(Some(FileInner { dir: None, file: None,std:1 ,path: String::new(), flag: 0 }));
                v.push(Some(FileInner { dir: None, file: None,std:2 ,path: String::new(), flag: 0 }));
                v.push(Some(FileInner { dir: None, file: None,std:3 ,path: String::new(), flag: 0 }));
                v
            },
            uthread:Box::new(UThread::empty())
        }
    }
    pub fn new_thread_by_elf(data:&[u8])->Thread{
        let elf=ElfFile::new(data).expect("illegal elf");
        assert!(elf.header.pt2.type_().as_type()==header::Type::Executable);
        let entry_addr=elf.header.pt2.entry_point() as usize;
        let mut thread=Thread::new_thread_same_pgtable();
        elf.add_memory_area(&mut thread.pgtable);
        thread.uthread=Box::new(UThread::new(entry_addr,thread.kthread.kstack.top_addr(),&mut thread.pgtable));
        thread
    }

}

#[macro_export]
macro_rules! my_thread {
    () => {
        THREAD_POOL.get_mut().pool[CURRENT_TID.lock()[cpu_id()]].lock().as_mut().unwrap().thread
    };
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
    pub fn insert(&mut self,mut thread:Box<Thread>)->Tid{
        let tid=self.alloc_tid();
        thread.tid=tid;
        *self.pool[tid].lock()=Some(
            ThreadInfo { thread, status: Status::Ready }
        );
        tid
        
    }
    pub fn remove(&mut self,tid:Tid){
        debug_assert!(self.pool[tid].lock().is_some());
        *self.pool[tid].lock()=None;
    }
    pub fn get_status(&self,tid:Tid)->Status{
        debug_assert!(self.pool[tid].lock().is_some());
        self.pool[tid].lock().as_mut().unwrap().status.clone()
    }
    pub fn change_status(&mut self,tid:Tid,status:Status){
        debug_assert!(self.pool[tid].lock().is_some());
        self.pool[tid].lock().as_mut().unwrap().status=status;
    }
}
