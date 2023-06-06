use core::{cell::UnsafeCell, iter::empty};

use super::{kthread::*, uthread::{UThread, TrapFrame}, scheduler::CURRENT_TID};
use crate::{config::*, lang_items::TrustCell, memory::{page_table::PageTable, map_kernel}, arch::cpu_id, timer::Tms, fs::fs::{File_, FileInner, OFlags}};
use alloc::{boxed::Box, vec::Vec, string::String};
use spin::*;
use xmas_elf::ElfFile;
use xmas_elf::header;
use crate::proc::elf::*;

pub type Tid=usize;
pub type ExitCode=isize;
lazy_static!{
    pub static ref THREAD_POOL:TrustCell<ThreadPool> =TrustCell::new(ThreadPool::new(MAX_THREAD_NUM));
}

#[derive(Clone,PartialEq, Eq)]
pub enum Status {
    Ready,
    Running,
    LightSleeping,//可中断
    DeepSleeping,//不可中断
    Killed(ExitCode),//ExitCode
}

pub struct Thread{
    pub tid:Tid,
    pub father_tid:Option<Tid>,
    pub child_tid:Vec<Tid>,
    pub pgtable:PageTable,
    pub tms:Tms,
    pub cwd:String,
    pub fd_table:Vec<Option<FileInner>>,
    pub kthread:Box<KThread>,
    pub uthread:Box<UThread>,//这实际上只在初始化时有效，之后，真实的tf已经在内核栈上了
}

pub struct ThreadInfo{
    pub thread:Box<Thread>,
    pub status:Status,
}

pub struct ThreadPool{
    pub pool:Vec<Mutex< Option<ThreadInfo> >>,
}

impl Thread {
    pub fn empty()->Thread{
        Thread { 
            tid: MAX_THREAD_NUM, 
            father_tid:None,
            child_tid:Vec::new(),
            pgtable:PageTable::empty() ,
            tms:Tms::empty(),
            cwd:String::new(),
            fd_table:Vec::new(),
            kthread: Box::new(KThread::empty()),
            uthread:Box::new(UThread::empty()) 
        }
    }

    pub fn fork(&self,sp:usize,mut tf:TrapFrame)->Thread{
        tf.x[10]=0;
        let mut new_thread=Thread::empty();
        new_thread.father_tid=Some(self.tid);
        new_thread.pgtable=self.pgtable.clone();
        new_thread.cwd=self.cwd.clone();
        new_thread.fd_table=self.fd_table.clone();
        new_thread.kthread=KThread::new_kthread(new_thread.pgtable.root_ppn);
        new_thread.uthread=Box::new(UThread::new_by_tf(new_thread.kthread.kstack.top_addr(),tf,tf.sepc,sp));
        new_thread
    }
    pub fn new_thread_same_pgtable()->Thread{
        let pgtable=map_kernel();
        let root_ppn=pgtable.root_ppn;
        Thread{
            tid:MAX_THREAD_NUM,
            father_tid:None,
            child_tid:Vec::new(),
            pgtable,
            tms:Tms::empty(),
            cwd:String::new(),
            kthread:KThread::new_kthread(root_ppn),
            fd_table:{
                let mut v=Vec::new();
                v.push(Some(FileInner { dir: None, file: None,std:1 ,o_flag:OFlags::O_RDONLY, path: String::new(), flag: 0 }));
                v.push(Some(FileInner { dir: None, file: None,std:2 ,o_flag:OFlags::O_WRONLY,path: String::new(), flag: 0 }));
                v.push(Some(FileInner { dir: None, file: None,std:3 ,o_flag:OFlags::O_WRONLY,path: String::new(), flag: 0 }));
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

#[macro_export]
macro_rules! my_lock {
    () => {
        THREAD_POOL.get_mut().pool[CURRENT_TID.lock()[cpu_id()]].lock()
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
        if thread.father_tid.is_some(){
            self.pool[thread.father_tid.unwrap()].lock().as_mut().unwrap().thread.child_tid.push(tid);
        }
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
