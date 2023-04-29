use crate::sbi::putchar;
use core::fmt::{self, Write};
pub struct Stdout;



pub static STDOUT_MUTEX:spin::Mutex<Stdout>=spin::Mutex::new(Stdout);


impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    STDOUT_MUTEX.lock().write_fmt(args).unwrap();
}

//强制输出。会尝试5000次获取print锁，如果还不能成功，则认为可能发生死锁，此时强制解锁并获取锁
pub fn force_print(args: fmt::Arguments){
    let mut times = 0;
    let mut stdout=STDOUT_MUTEX.try_lock();
    while stdout.is_none(){
        times+=1;
        stdout=STDOUT_MUTEX.try_lock();
        if times >= 5000 {
            break;
        }
    }
    while stdout.is_none(){
        unsafe{STDOUT_MUTEX.force_unlock()};
        stdout=STDOUT_MUTEX.try_lock();
    }
    stdout.unwrap().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::stdio::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::stdio::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! eprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::stdio::force_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


