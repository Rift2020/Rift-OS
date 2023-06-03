#![feature(panic_info_message)]
#![allow(unused)]

use core::{arch::asm, panicking::panic};
//const EID_SET_TIMER: isize = 0;
const EID_CONSOLE_PUTCHAR: isize = 1;
const EID_CONSOLE_GETCHAR: isize = 2;
const EID_CLEAR_IPI: isize = 3;
const EID_SEND_IPI: isize = 4;
const EID_REMOTE_FENCE_I: isize = 5;
const EID_REMOTE_SFENCE_VMA: isize = 6;
const EID_REMOTE_SFENCE_VMA_ASID: isize = 7;
const EID_SHUTDOWN: isize = 8;

const EID_BASE:isize=0x10;
const FID_GET_SBI_IMPLEMENTATION_ID:isize=1;

const EID_HSM:isize=0x48534D;//Hart State Management
const FID_HART_START:isize=0x0;

const EID_TIMER:isize = 0x54494D45;
const FID_SET_TIMER:isize = 0;

#[inline(always)]
fn sbi_call(extension_id: isize ,function_id: isize, arg0: usize, arg1: usize, arg2: usize) -> (isize,isize) {
    let mut error;
    let mut value;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => error,
            inlateout("x11") arg1 => value,
            in("x12") arg2,
            in("x16") function_id,
            in("x17") extension_id,
        );
    }
    (error,value)
}


pub fn putchar(c:usize){
    sbi_call(EID_CONSOLE_PUTCHAR,0,c,0,0);
}
pub fn shutdown()->!{
    sbi_call(EID_SHUTDOWN,0,0,0,0);
    panic!("shutdown fail!");
}
pub fn get_sbi_implementation_id()->isize{
    sbi_call(EID_BASE,FID_GET_SBI_IMPLEMENTATION_ID,0,0,0).1
}

//启动指定hartid的核心，它将会从start_addr(PA)开始执行，且此时其a1寄存器被设置为opaque的值
pub fn hart_start(hartid:usize,start_addr:usize,opaque:usize)->isize{
    sbi_call(EID_HSM,FID_HART_START,hartid,start_addr,opaque).0
}

pub fn set_timer(stime_value:usize)->isize{
    sbi_call(EID_TIMER,FID_SET_TIMER,stime_value,0,0).0
}
