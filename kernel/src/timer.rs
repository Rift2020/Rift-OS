use core::ops::Add;

use riscv::register::{cycle, time};
use spin::Mutex;

use crate::board::CPU_FREQ;
use crate::config::{TIME_INTERRUPT_PER_SEC,CPU_NUM};
use crate::sbi::set_timer;


pub const TIME_INTERRUPT_CYCLES:usize=CPU_FREQ/TIME_INTERRUPT_PER_SEC;

//记录CYCLE数值
pub static LAST_CYCLE:Mutex<[usize;CPU_NUM]> =Mutex::new([0;CPU_NUM]);
//通过trap开始，到(trap结束 或 yield开始)，(yield结束到trap结束，中间的时间就是该线程内核态时间
//而从trap结束，到下一个trap开始，中间的时间就是该线程用户态的时间

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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TimeVal {
    pub tv_sec: usize,  //seconds
    pub tv_usec: usize, //microseconds
}

impl TimeVal {
    pub fn get_timeval()->TimeVal{
        let cycle=get_cycle();
        let sec=cycle/CPU_FREQ;
        let usec=cycle%CPU_FREQ*1000/CPU_FREQ;
        TimeVal { tv_sec: sec, tv_usec: usec }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug,PartialEq, Eq,PartialOrd, Ord)]
pub struct TimeSpec{
    pub tv_sec :usize,
    pub tv_nsec:usize,
}



impl TimeSpec {
    pub fn get_timespec()->TimeSpec{
        let cycle=get_cycle();
        let sec=cycle/CPU_FREQ;
        let nsec=cycle%CPU_FREQ*1000_000_000/CPU_FREQ;
        TimeSpec { tv_sec: sec, tv_nsec: nsec }
    }
}

impl Add for TimeSpec {
    type Output = TimeSpec;
    fn add(self, rhs: Self) -> Self::Output {
        let sec=self.tv_sec+rhs.tv_sec+(self.tv_nsec+rhs.tv_nsec)/1000_000_000;
        let nsec=(self.tv_nsec+rhs.tv_nsec)%1000_000_000;
        TimeSpec { tv_sec: sec, tv_nsec: nsec }
    }
}

pub fn get_cycle()->usize{
    cycle::read()
}

pub fn get_time()->usize{
    time::read()
}

pub fn set_next_time_interrupt(){
    set_timer(time::read()+TIME_INTERRUPT_CYCLES);
}
