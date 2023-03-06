#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_panic)]
#![feature(fn_align)]

extern crate alloc;

mod lang_items;
mod sbi;
#[macro_use]
mod stdio;
mod config;
mod memory;
mod interrupt;

use core::arch::global_asm;
use core::arch::asm;
use riscv;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Rift os is booting");
    memory::init();
    memory::test();
    interrupt::init_interrupt();
    unsafe{
        riscv::asm::ebreak();
    }
    panic!("should panic in kernel_trap");
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
