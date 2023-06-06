use crate::arch::cpu_id;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::{self, THREAD_POOL,Status};

pub fn sys_exit(ec:isize){
    let tid=my_thread!().tid;
    THREAD_POOL.get_mut().change_status(tid,Status::Killed(ec));
    yield_();
}
