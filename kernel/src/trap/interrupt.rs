use core::arch::global_asm;
use riscv::register::{
    scause,
    sepc,
    stvec,
    sscratch,
    stval,
};

use crate::arch::cpu_id;

global_asm!(include_str!("trap.asm"));

pub fn init_interrupt() {
    extern "C"{
        fn __alltraps();
    }
    unsafe {
        sscratch::write(0);
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
    println!("[Rift os] init_interrupt!");
}

#[no_mangle]
fn kernel_trap() -> ! {
    let cause = scause::read().cause();
    let epc = sepc::read();
    eprintln!("kernel_trap[CPU{}]: cause: {:?}, epc: {:#x}",cpu_id(),cause , epc);
    if scause::read().bits()==15 {
        eprintln!("StorePageFault in :{:#x}",stval::read());
    }
    panic!("trap handled!");
}
