use core::mem::size_of;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::fs::FILE_SYSTEM;
use crate::memory::address::*;

use crate::arch::cpu_id;
use crate::my_lock;
use crate::my_thread;
use crate::proc::kthread::yield_;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::scheduler::GLOBAL_SCHEDULER;
use crate::proc::scheduler::Scheduler;
use crate::proc::thread::*;
use crate::memory::page_table::*;
use crate::proc::uthread::TrapFrame;

use super::get_user_string;
use super::user_buf_to_vptr;

pub fn sys_clone(flags:usize,child_stack:usize,ptid:usize,tls:usize,ctid:usize,tf:TrapFrame)->isize{
    if flags!=17{
        panic!("sys_clone:otherflags TODO");
    }
    //println!("cs {}",child_stack);
    let mut child_stack=child_stack;
    if child_stack==0{
        child_stack=tf.x[2];
    }
    let mut cthread=Box::new(my_thread!().fork(child_stack,tf));
    //println!("tf {:?} {:?}",tf,cthread.uthread.trapframe);
    //println!("kstack{:#x} {:#x}",my_thread!().kthread.context_addr,cthread.kthread.context_addr);
    let ret=GLOBAL_SCHEDULER.lock().push_thread(cthread) as isize;
    //println!("ret:{}",ret);
    //println!("len:{}",my_thread!().child_tid.len());
    ret
}

pub fn sys_execve(path:*const u8,argv:*const usize,envp:*const usize)->isize{
    //envp 未实现
    let mut _path=match get_user_string(path) {
        None=>{
           println!("path string error");
            return -1;
        }
        Some(s)=>s,
    };
    let mut data=Box::new([0u8;4096*512]);
    match FILE_SYSTEM.root_dir().open_file(&_path) {
        Ok(file)=>{
            file.read(data.as_mut()).ok().unwrap();
        }
        Err(e)=>{
            println!("wrong path {}",_path);
            return -1;
        }
    }
    FILE_SYSTEM.root_dir().open_file(&_path).unwrap().read(data.as_mut()).ok().unwrap();
    let mut args:Vec<String>=Vec::new();
    //缺失安全检查
    let mut argv=argv;
    if !argv.is_null(){
        loop {
            let str_uptr_ptr=my_thread!().pgtable.uva_to_kusize(VirtAddr::from(argv as usize)) as *const usize;
            let str_uptr=unsafe {*str_uptr_ptr} as *const u8;
            if str_uptr.is_null(){
                break;
            }
            let str1=get_user_string(str_uptr).unwrap();
            //println!("argv {},sup {},su {} str1 {}",argv as usize,str_uptr_ptr as usize,str_uptr as usize,str1);
            args.push(get_user_string(str_uptr).unwrap());
            unsafe {argv=argv.add(1);}
        }
    }
    //println!("Vec len: {}",args.len());
    //for i in &args{
    //    println!("vec: {}",i);
    //}
    my_thread!().exec_by_elf(data.as_ref(),args);
    0
}

pub fn sys_wait(pid:isize,status:usize,opt:usize)->isize{
    if pid==-1{
        loop {
            let len=my_thread!().child_tid.len();
            if len==0{
                return -1;
            }
            for i in 0..len{
                let ctid=my_thread!().child_tid[i];
                let stat=THREAD_POOL.get_mut().pool[ctid].lock().as_ref().unwrap().status.clone();
                if let Status::Killed(exitcode)=stat{
                    THREAD_POOL.get_mut().remove(ctid);
                    my_thread!().child_tid.remove(i);
                    let vpt=user_buf_to_vptr(status,size_of::<i32>(),PTEFlags::W);
                    match vpt {
                        None=>return -1,
                        Some(p)=>{
                            let ptr=p as *mut i32;
                            unsafe {
                                *ptr=(exitcode<<8) as i32;
                            }
                        }
                    }
                    return ctid as isize;
                }
            }
            yield_();
        }
    }
    else{
        let mut ctid=-1;
        let mut ind=0;
        let len=my_thread!().child_tid.len();
        if len==0{
            return -1;
        }
        for i in 0..len{
            if my_thread!().child_tid[i]==pid as usize {
                ctid=pid;
                ind=i;
                break;
            }
        }
        if ctid==-1{
            return -1;
        }
        let ctid=ctid as usize;
        loop {
            let stat=THREAD_POOL.get_mut().pool[ctid].lock().as_ref().unwrap().status.clone();
            if let Status::Killed(exitcode)=stat{
                THREAD_POOL.get_mut().remove(ctid);
                my_thread!().child_tid.remove(ind);
                let vpt=user_buf_to_vptr(status,size_of::<i32>(),PTEFlags::W);
                    match vpt {
                        None=>return -1,
                        Some(p)=>{
                            let ptr=p as *mut i32;
                            unsafe {
                                //感谢bite_the_disk的代码启示了我，不然我怎么也想不到还有24位这种逆天数据
                                *ptr=(exitcode<<8) as i32;
                            }
                        }
                    }

                return ctid as isize;
            }
            yield_();
        }
    }
    -1
}
