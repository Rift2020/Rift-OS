use riscv::register::{
    scause,
    sepc,
    stvec,
    sscratch
};

pub fn init_interrupt() {
    unsafe {
        sscratch::write(0);
        stvec::write(kernel_trap as usize, stvec::TrapMode::Direct);
    }
    println!("[Rift os] init_interrupt!");
}

#[repr(align(4))]
fn kernel_trap() -> ! {
    let cause = scause::read().cause();
    let epc = sepc::read();
    println!("kernel_trap: cause: {:?}, epc: 0x{:#x}", cause, epc);
    panic!("trap handled!");
}
