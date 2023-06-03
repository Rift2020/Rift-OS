use riscv::register::time;

use crate::board::CPU_FREQ;
use crate::config::TIME_INTERRUPT_PER_SEC;
use crate::sbi::set_timer;

const TIME_INTERRUPT_CYCLES:usize=CPU_FREQ/TIME_INTERRUPT_PER_SEC;

pub struct Tms {
    pub tms_utime: isize,  // User CPU time
    pub tms_stime: isize,  // System CPU time
    pub tms_cutime: isize,  // User CPU time of dead children
    pub tms_cstime: isize,	// System CPU time of dead children
}

pub fn get_time_val()->usize{
    time::read()
}

pub fn set_next_time_interrupt(){
    set_timer(time::read()+TIME_INTERRUPT_CYCLES);
}
