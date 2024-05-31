use core::mem::zeroed;

use alloc::string::String;
use alloc::vec::Vec;
use riscv::register::{
    sstatus,
    sstatus::*,
    scause::Scause,
};
use crate::arch::cpu_id;
use crate::memory::address::VirtAddr;
use crate::memory::page_table::PTEFlags;
use crate::my_thread;
use crate::proc::kthread::Context;

use crate::{memory::{page_table::{PageTable, self}, frame::{FrameArea, FrameFlags}, address::VirtPageNum}, config::{USER_STACK_SIZE, PAGE_SIZE, USER_STACK_TOP}};

use super::kthread::forkret;


#[repr(C)]
#[derive(Clone, Copy,Debug)]
pub struct TrapFrame {
    pub x: [usize; 32], // General registers
    pub sstatus: usize, // Supervisor Status Register
    pub sepc: usize, // Supervisor exception program counter
    pub tp:usize,//这是为内核线程服务的，由于用户程序可能会修改tp寄存器，所以我们将tp寄存器（在内核中，他表示当前的cpu_id）保存在tf中，当且仅当初始化时赋值为cpu_id以及每次trap时用来恢复tp寄存器。
    //由于trap.asm为硬编码寄存器地址，上面的变量顺序不可随意调整，如需调整，需要重新修改asm代码

}

pub struct UThread{
    pub trapframe:TrapFrame,//TODO:可能需要修改
}

impl UThread {
    pub fn empty()->UThread {
        unsafe{
            zeroed()
        }
    }
    pub fn new(entry_addr:usize,kstack_top_addr:usize,pgtable:&mut PageTable)->UThread{
        let ustack=FrameArea::new_by_alloc(USER_STACK_SIZE/PAGE_SIZE,FrameFlags::R|FrameFlags::W|FrameFlags::U).unwrap();
        pgtable.map(VirtPageNum::from(VirtAddr::from(USER_STACK_TOP-USER_STACK_SIZE)),ustack);

        let mut uthread=UThread::empty();
        //println!("cpu id {}",cpu_id());
        uthread.trapframe.x[4]=cpu_id();
        uthread.trapframe.x[2]=USER_STACK_TOP-16000;//暂时不清楚为什么需要减一点
                                                 //似乎user
                                                 //sp既向上又向下移动，但是按道理不应该是这样
        uthread.trapframe.sepc=entry_addr;

        uthread.trapframe.sstatus=0x8000_0002_0000_6000;//权宜之计，本来应该是在这里读取sstatus，现在手动给他赋上去
        //uthread.trapframe.sstatus|=0x20;
        //uthread.trapframe.sstatus&=0xFFFF_FFFF_FFFF_FFFD;
        uthread.trapframe.sstatus&=0xFFFF_FFFF_FFFF_FEFF;//使得sret会进入用户态
        uthread.trapframe.tp=cpu_id();
        
        unsafe{uthread.push_at_tf(kstack_top_addr);}
        uthread
    } 
    pub fn new_with_args(entry_addr:usize,kstack_top_addr:usize,pgtable:&mut PageTable,args:Vec<String>)->UThread{
        //println!("new_with_args start");
        let mut uthread=Self::new(entry_addr,kstack_top_addr,pgtable);

        //不同于常规的处理方式，传给用户argc,argv用的是a2,a3寄存器而非a1,a2
        uthread.trapframe.x[11]=args.len();
        uthread.trapframe.x[12]=uthread.trapframe.x[2];
        unsafe{uthread.push_at_tf(kstack_top_addr);}
        //_
        assert!(pgtable.check_user_range(VirtAddr::from(uthread.trapframe.x[12]),VirtAddr::from(uthread.trapframe.x[12]+10000),PTEFlags::U )==true);
        let mut argv=pgtable.uva_to_kusize(VirtAddr::from(uthread.trapframe.x[12])) as *mut usize;
        let mut str_uptr=(USER_STACK_TOP-8000) as *const u8;
        let mut str_ptr=pgtable.uva_to_kusize(VirtAddr::from(USER_STACK_TOP-8000)) as *mut u8; 
        for i in args{
            //println!("argv {:#x},uprt {:#x}, str {:#x}",argv as usize,str_uptr as usize,str_ptr as usize);
            //println!("string:{}",i);
            unsafe{
                *argv=str_uptr as usize;
//                assert!(pgtable.check_user_range(VirtAddr::from(argv),VirtAddr::from(uthread.trapframe.x[12]+10000),PTEFlags::U )==true);
                for c in i.as_bytes(){
                    *str_ptr=*c;
                    str_uptr=str_uptr.add(1);
                    str_ptr=str_ptr.add(1);
                }
                *str_ptr=0;
                str_uptr=str_uptr.add(1);
                str_ptr=str_ptr.add(1);
                argv=argv.add(1);
            }
        }
        unsafe{*argv=0};
        //println!("argv {},uprt {}, str {}",argv as usize,str_uptr as usize,str_ptr as usize);
        
        //_
        //println!("tp:{}",uthread.trapframe.tp);
        uthread
    }
    unsafe fn push_at_tf(&self,stack_top_addr:usize)->usize{
        let ptr=(stack_top_addr as *mut TrapFrame).sub(1);
        *ptr=self.trapframe;
        ptr as usize
    }
    pub fn new_by_tf(kstack_top_addr:usize,tf:TrapFrame,sepc:usize,sp:usize)->UThread{
        let mut new_uthread=UThread{trapframe:tf.clone()};
        new_uthread.trapframe.sepc=sepc;
        new_uthread.trapframe.x[2]=sp;
        unsafe{new_uthread.push_at_tf(kstack_top_addr);}
        new_uthread
    }
}
