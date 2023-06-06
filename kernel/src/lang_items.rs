use core::{panic::PanicInfo, cell::UnsafeCell};
use crate::{eprintln, sbi};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        eprintln!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        eprintln!("Panicked: {}", info.message().unwrap());
    }
    loop{

    }
    sbi::shutdown();
}

//不提供任何多线程下的保护，但能通过编译
//应该以其他方式保证其中数据线程安全
pub struct TrustCell<T>{
    data:UnsafeCell<T>,
}

unsafe impl<T> Sync for TrustCell<T> {
    //什么都没有
}

impl<T> TrustCell<T> {
    pub fn new(value:T)->Self{
        Self { data: UnsafeCell::new(value) }
    }
    pub fn get_mut(&self)->&mut T{
        unsafe{ &mut(*self.data.get())}
    }
}
