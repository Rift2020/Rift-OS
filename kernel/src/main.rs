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

use core::arch::global_asm;
use core::arch::asm;
use riscv;

use crate::proc::kthread::*;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[Rift os] booting");
    trap::interrupt::init_interrupt();
    let pgtable=memory::init();
    //pgtable.print();
    pgtable.set_satp_to_root();
    memory::test();
    /*
    println!("interrupt test");
    unsafe{
        riscv::asm::ebreak();
    }
    panic!("should panic in kernel_trap");
    */
    println!("Are you there? kthread2 !");
    let mut kthread2:alloc::boxed::Box<KThread>=KThread::new_kthread_same_pgtable();
    let mut context=Context::empty();
    unsafe{
        INIT_KTHREAD.context_addr=&mut context as *mut Context as usize;
        INIT_KTHREAD.switch_to(&mut kthread2);
    }
    println!("kthread1 is back ! now shutdown !");
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
