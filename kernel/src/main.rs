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
#[path = "arch/riscv/mod.rs"]
mod arch;

use core::arch::global_asm;
use core::arch::asm;
use alloc::vec::Vec;
use riscv;

use crate::arch::cpu_id;
use crate::arch::start_cpu_from_start2;
use crate::config::CPU_NUM;
use crate::config::PHYS_KERNEL_START;
use crate::config::PHYS_VIRT_OFFSET;
use crate::proc::kthread::*;
use crate::sbi::hart_start;
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
        
        let mut kstack_vec:Vec<KStack>=Vec::new();
        for i in 0..CPU_NUM{
            if i==cpu_id(){
                continue;
            }
            kstack_vec.push(start_cpu_from_start2(i));
        }
        memory::test(); 

        let mut kthread2:alloc::boxed::Box<KThread>=KThread::new_kthread_same_pgtable();
        let mut context=Context::empty();
        println!("switch test");
        unsafe{
            INIT_KTHREAD.context_addr=&mut context as *mut Context as usize;
            INIT_KTHREAD.switch_to(&mut kthread2);
        }
        println!("kthread1 is back !"); 
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
