use core::arch::global_asm;
use riscv::register::{
    scause::{
        self,
        Trap,
        Exception,
    },
    sepc,
    stvec,
    sscratch,
    stval,
};
use crate::proc::uthread::TrapFrame;
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

fn call_syscall(tf: &mut TrapFrame) {
    // 返回后跳转到 ecall 下一条指令
    tf.sepc += 4;
    let ret = crate::syscall::syscall(
       tf.x[17],
        [tf.x[10], tf.x[11], tf.x[12], tf.x[13], tf.x[14], tf.x[15]]
    );
    tf.x[10] = ret as usize;
}

#[no_mangle]
fn trap(tf: &mut TrapFrame) {
    let cause = scause::read().cause();
    let epc = sepc::read();
    let sscratch=sscratch::read();
    match cause {
        Trap::Exception(Exception::UserEnvCall) => call_syscall(tf),
        Trap::Exception(Exception::StorePageFault)=>{
            eprintln!("StorePageFault in :{:#x}",stval::read());
            panic!("trap handled!")

        },
        _ => {
            println!("kernel_trap[CPU{}]: cause: {:?}, epc: {:#x}",cpu_id(),cause , epc);
            panic!("trap handled!");
        }
    }
}
