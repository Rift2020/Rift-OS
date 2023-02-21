#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(core_panic)]

mod lang_items;
mod sbi;
mod stdio;

use core::arch::global_asm;


global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Rift os is booting");
    sbi::shutdown();
}

fn clear_bss() {
    extern "C" {
        static sbss: u64;
        static ebss: u64;
    }
    unsafe {
        (sbss..ebss).for_each(|a| {
            (a as *mut u8).write_volatile(0)
        });
    }
}
