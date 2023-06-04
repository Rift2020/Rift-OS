use riscv::register::time;
use spin::Mutex;

use crate::board::CPU_FREQ;
use crate::config::{TIME_INTERRUPT_PER_SEC,CPU_NUM};
use crate::sbi::set_timer;


pub const TIME_INTERRUPT_CYCLES:usize=CPU_FREQ/TIME_INTERRUPT_PER_SEC;
pub static LAST_CYCLE:Mutex<[usize;CPU_NUM]> =Mutex::new([0;CPU_NUM]);

#[repr(C)]
#[derive(Clone, Copy,Debug)]
pub struct Tms {
    pub tms_utime: isize,  // User CPU time
    pub tms_stime: isize,  // System CPU time
    pub tms_cutime: isize,  // User CPU time of dead children
    pub tms_cstime: isize,	// System CPU time of dead children
}

impl Tms {
    pub const fn empty()->Tms{
        Tms { tms_utime: 0, tms_stime: 0, tms_cutime: 0, tms_cstime: 0 }
    }
}

pub fn get_cycle()->usize{
    time::read()
}

pub fn set_next_time_interrupt(){
    set_timer(time::read()+TIME_INTERRUPT_CYCLES);
}
