pub mod time;
use core::mem::size_of;

use alloc::string::String;
use crate::memory::address::VirtAddr;

use crate::arch::cpu_id;
use crate::my_thread;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::*;
use crate::memory::page_table::*;

const _UTSNAME_LENGTH:usize=65;
#[derive(Clone, Copy)]
pub struct Utsname
{
    /* Name of the implementation of the operating system.  */
    pub sysname:[u8;_UTSNAME_LENGTH],

    /* Name of this node on the network.  */
    pub nodename:[u8;_UTSNAME_LENGTH],

    /* Current release level of this implementation.  */
    pub release:[u8;_UTSNAME_LENGTH],

    /* Current version level of this release.  */
    pub version:[u8;_UTSNAME_LENGTH],

    /* Name of the hardware type the system is running on.  */
    pub machine:[u8;_UTSNAME_LENGTH],

    pub domainname:[u8;_UTSNAME_LENGTH]
}

fn gen_by_str(s:&str)->[u8;65]{
    let mut ret=['\0' as u8;65];
    ret[..s.len()].copy_from_slice(s.as_bytes());
    ret
}
/*
lazy_static!{
    #[derive(Clone, Copy)]
    pub static ref UTSNAME:Utsname=Utsname{
        sysname:gen_by_str("Rift-OS"),
        nodename:gen_by_str("root"),
        release:gen_by_str("0.0.0"),
        version:gen_by_str("0.0.0"),
        machine:gen_by_str("QEMU (RISC-V 64)"),
        domainname:gen_by_str("https://github.com/Rift2020/Rift-OS"),
    };
}
*/

pub fn sys_uname(buf:*mut Utsname)->isize{
    let vstart=VirtAddr::from(buf as usize);
    let vend=VirtAddr::from(buf as usize+size_of::<Utsname>());
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::W)==false{
        return -1;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *mut Utsname;
    unsafe{
        *vpt=Utsname{
            sysname:gen_by_str("Rift-OS"),
            nodename:gen_by_str("root"),
            release:gen_by_str("0.0.0"),
            version:gen_by_str("0.0.0"),
            machine:gen_by_str("QEMU (RISC-V 64)"),
            domainname:gen_by_str("https://github.com/Rift2020/Rift-OS"),
        };
    }
    return 0;
}
