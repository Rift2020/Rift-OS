#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_panic)]
#![feature(fn_align)]
#![feature(naked_functions)]


extern crate alloc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

mod lang_items;
mod sbi;
#[macro_use]
mod stdio;
mod config;
mod memory;
mod trap;
mod proc;
mod driver;
#[path ="board/qemu.rs"]
mod board;
#[path = "arch/riscv/mod.rs"]
mod arch;


use core::arch::global_asm;
use core::arch::asm;
use alloc::boxed::Box;
use alloc::vec::Vec;
use riscv;

use crate::arch::cpu_id;
use crate::arch::start_cpu_from_start2;
use crate::config::CPU_NUM;
use crate::proc::kthread::*;
use crate::proc::thread;
use crate::proc::thread::*;
use crate::proc::scheduler::*;
use core::sync::atomic::{AtomicBool, Ordering};


global_asm!(include_str!("entry.asm"));

pub static INIT_HART:AtomicBool=AtomicBool::new(true);

#[no_mangle]
pub fn rust_main() -> ! { 
    //首个启动的hart
    if INIT_HART.compare_exchange(true,false,Ordering::Relaxed,Ordering::Relaxed)==Ok(true){
        clear_bss();
        println!("[Rift os] booting");
        println!("hart {} starting",cpu_id());
        
        trap::interrupt::init_interrupt();
        
        let pgtable=memory::init();
        pgtable.set_satp_to_root();
        println!("start other hart");
        let mut kstack_vec:Vec<KStack>=Vec::new();
        for i in 0..CPU_NUM{
            if i==cpu_id(){
                continue;
            }
            kstack_vec.push(start_cpu_from_start2(i));
        }
        memory::test();
        
        //我是IDLE
        let mut idle_thread:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        idle_thread.pgtable=pgtable;
        let idle_tid=idle_thread.tid;
        IDLE_TID.lock()[cpu_id()]=idle_thread.tid;
        CURRENT_TID.lock()[cpu_id()]=idle_thread.tid;
        THREAD_POOL.get_mut().insert(idle_thread);

        driver::block_device::block_device_test();
        
        let thread2:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        let thread2_tid=thread2.tid;
        GLOBAL_SCHEDULER.lock().push_thread(thread2);
    
        let thread3:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        GLOBAL_SCHEDULER.lock().push_thread(thread3);

        for i in 0..5 {
            println!("hi i'm scheduler(cpuid:{})",cpu_id());
            let next_tid=GLOBAL_SCHEDULER.lock().pop().unwrap();
            THREAD_POOL.get_mut().pool[idle_tid].lock().as_mut().unwrap().thread.kthread.switch_to(next_tid);
            GLOBAL_SCHEDULER.lock().push_tid(next_tid);
        }

        loop {
        
        }

    }
    //除了首个启动的核心，其他核心进入rust_main后将会直接执行该分支
    else{
        println!("hart {} starting",cpu_id());
        trap::interrupt::init_interrupt();
        let pgtable=memory::map_kernel();
        pgtable.set_satp_to_root();
        memory::test(); 
        
        //我是IDLE
        let mut idle_thread:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        idle_thread.pgtable=pgtable;
        let idle_tid=idle_thread.tid;
        IDLE_TID.lock()[cpu_id()]=idle_thread.tid;
        CURRENT_TID.lock()[cpu_id()]=idle_thread.tid;
        THREAD_POOL.get_mut().insert(idle_thread);

        driver::block_device::block_device_test();
        
        let thread4:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        GLOBAL_SCHEDULER.lock().push_thread(thread4);
    
        let thread5:Box<Thread>=Box::new(Thread::new_thread_same_pgtable());
        GLOBAL_SCHEDULER.lock().push_thread(thread5);
        
        for i in 0..5 {
            println!("hi i'm scheduler(cpuid:{})",cpu_id());
            let next_tid=GLOBAL_SCHEDULER.lock().pop().unwrap();
            THREAD_POOL.get_mut().pool[idle_tid].lock().as_mut().unwrap().thread.kthread.switch_to(next_tid);
            GLOBAL_SCHEDULER.lock().push_tid(next_tid);
        }
        println!("all test passed");
        loop {
            
        }
    }


    /*
    println!("interrupt test");
    unsafe{
        riscv::asm::ebreak();
    }
    panic!("should panic in kernel_trap");
    */
     sbi::shutdown();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        let sbss=sbss as usize;
        let ebss=ebss as usize;
        (sbss..ebss).for_each(|a| {
            (a as *mut u8).write_volatile(0)
        });
    }
}
