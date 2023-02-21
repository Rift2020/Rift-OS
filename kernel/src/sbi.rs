#![feature(panic_info_message)]
#![allow(unused)]

use core::{arch::asm, panicking::panic};
const EID_SET_TIMER: i32 = 0;
const EID_CONSOLE_PUTCHAR: i32 = 1;
const EID_CONSOLE_GETCHAR: i32 = 2;
const EID_CLEAR_IPI: i32 = 3;
const EID_SEND_IPI: i32 = 4;
const EID_REMOTE_FENCE_I: i32 = 5;
const EID_REMOTE_SFENCE_VMA: i32 = 6;
const EID_REMOTE_SFENCE_VMA_ASID: i32 = 7;
const EID_SHUTDOWN: i32 = 8;

#[inline(always)]
fn sbi_call(extension_id: i32 ,function_id: i32, arg0: u32, arg1: u32, arg2: u32) -> (i32,i32) {
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

pub fn putchar(c:u32){
    sbi_call(EID_CONSOLE_PUTCHAR,0,c,0,0);
}
pub fn shutdown()->!{
    sbi_call(EID_SHUTDOWN,0,0,0,0);
    panic!("shutdown fail!");
}
