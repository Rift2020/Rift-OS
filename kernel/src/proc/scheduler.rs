use spin::*;
use crate::config::{MAX_TICK, CPU_NUM, MAX_THREAD_NUM};
use super::thread::{*, self};
use alloc::{collections::{LinkedList, VecDeque}, boxed::Box};

pub static GLOBAL_SCHEDULER:Mutex<RRScheduler>=Mutex::new(RRScheduler::new(MAX_TICK));

pub static IDLE_TID:Mutex<[Tid;CPU_NUM]>= Mutex::new([MAX_THREAD_NUM,CPU_NUM]);
pub static CURRENT_TID:Mutex<[Tid;CPU_NUM]>= Mutex::new([MAX_THREAD_NUM,CPU_NUM]);

pub trait Scheduler {
    fn push_thread(&mut self,thread:Box<Thread>);//向调度器和线程池都push一个新线程
    fn push_tid(&mut self,tid:Tid);//push一个已经在线程池的线程
    fn pop(&mut self)->Option<Tid>;
    fn tick(&mut self)->bool;
    fn remove_scheduler(&mut self,tid:Tid);
    fn remove_thread(&mut self,tid:Tid);
}

pub struct RRScheduler{
    queue:VecDeque<Tid>,
    max_tick:usize,
    current_tick:usize,
}

impl RRScheduler{
   pub const fn new(max_tick:usize)->Self{
       Self{
           queue:VecDeque::new(),
           max_tick,
           current_tick:0,
       }
   } 
}

impl Scheduler for RRScheduler {
    fn push_thread(&mut self,thread:Box<Thread>) {
        self.queue.push_front(thread.tid);
        THREAD_POOL.get_mut().insert(thread);
    }           
    fn push_tid(&mut self,tid:Tid) {
        debug_assert!(THREAD_POOL.get_mut().pool[tid].lock().is_some());
        self.queue.push_back(tid);
    }
    fn pop(&mut self)->Option<Tid> {
        if self.queue.is_empty(){
            return None;
        }
        self.queue.pop_front()
    }
    fn tick(&mut self)->bool {
        self.current_tick+=1;
        if self.current_tick==self.max_tick{
            self.current_tick=0;
            return true;
        }
        return false;
    }
    fn remove_thread(&mut self,tid:Tid) {
        debug_assert!(THREAD_POOL.get_mut().pool[tid].lock().is_some());
        debug_assert!(self.queue.contains(&tid));
        self.queue.retain(|&x| x!=tid);
        debug_assert!(self.queue.contains(&tid)==false);
        *THREAD_POOL.get_mut().pool[tid].lock()=None;
    }
    fn remove_scheduler(&mut self,tid:Tid) {
        debug_assert!(self.queue.contains(&tid));
        self.queue.retain(|&x| x!=tid);
        debug_assert!(self.queue.contains(&tid)==false);

    }
}

