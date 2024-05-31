use core::arch::global_asm;
use riscv::register::{
    scause::{
        self,
        Trap,
        Exception,
        Interrupt,
    },
    sepc,
    stvec,
    sscratch,
    stval, sstatus,
};
use crate::{proc::{uthread::TrapFrame, kthread::yield_, scheduler::CURRENT_TID, thread::THREAD_POOL}, timer::{set_next_time_interrupt,get_cycle, LAST_CYCLE, self}, my_thread};
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
        [tf.x[10], tf.x[11], tf.x[12], tf.x[13], tf.x[14], tf.x[15]],
        tf.clone()
    );
    tf.x[10] = ret as usize;
}
#[inline(never)]
fn add_utime(){
    my_thread!().tms.tms_utime+=((get_cycle()-LAST_CYCLE.lock()[cpu_id()]+timer::TIME_INTERRUPT_CYCLES/2)/timer::TIME_INTERRUPT_CYCLES) as isize;
    LAST_CYCLE.lock()[cpu_id()]=get_cycle();
}
#[inline(never)]
fn add_stime(){
    my_thread!().tms.tms_stime+=((get_cycle()-LAST_CYCLE.lock()[cpu_id()]+timer::TIME_INTERRUPT_CYCLES/2)/timer::TIME_INTERRUPT_CYCLES) as isize;
    LAST_CYCLE.lock()[cpu_id()]=get_cycle();
}

#[no_mangle]
fn trap(tf: &mut TrapFrame) {
    //println!("tf:{:?}",tf);
    //println!("trap tp:{}",tf.tp);
    unsafe{sstatus::clear_sie();}//进内核态关中断，出内核态开中断(不是严格的边界)，但理论上只要开中断的时候不持有锁就可以，TODO:把锁换成自动开关中断的
    
    add_utime();
    let cause = scause::read().cause();
    let epc = sepc::read();
    let sscratch=sscratch::read();
    match cause {
        Trap::Exception(Exception::UserEnvCall) => call_syscall(tf),
        Trap::Interrupt(Interrupt::UserTimer)=>{//实际上用户态定时器中断也是走向SupervisorTimer
            panic!("This is impossible");
        }
        Trap::Interrupt(Interrupt::SupervisorExternal)=>{
            println!("external");
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {//用户态下，触发定时器中断就进行yield，以时间片轮转
            set_next_time_interrupt();
            yield_();
        }
        _ => {
            eprintln!("kernel_trap[CPU{}]: cause: {:?}, epc: {:#x}",cpu_id(),cause , epc);
            eprintln!("stval:{:#x}",stval::read());
            panic!("trap handled!");
        }
    }
    add_stime();
    unsafe{sstatus::set_sie();}
}
