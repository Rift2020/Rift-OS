use alloc::string::String;
use alloc::string::ToString;

use crate::fs::fs::*;
use crate::fs::FILE_SYSTEM;
use core::clone;
use core::mem::size_of;
use core::ptr::slice_from_raw_parts;
use core::ptr::slice_from_raw_parts_mut;
use crate::memory::address::VirtAddr;

use crate::arch::cpu_id;
use crate::my_thread;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::*;
use crate::memory::page_table::*;

use super::get_user_string;
use super::user_buf_to_vptr;
pub fn sys_getcwd(buf:*mut u8,size:usize)->isize{
    if buf as usize ==0 {
        println!("not impl for now");
        return 0;
    }
    let vstart=VirtAddr::from(buf as usize);
    let vend=VirtAddr::from(buf as usize+size_of::<u8>()*size);
    if my_thread!().pgtable.check_user_range(vstart,vend,PTEFlags::W)==false{
        return 0;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *mut u8;
    let cwd=String::from("/")+my_thread!().cwd.as_str()+"\0";
    if cwd.len()>size{
        return 0;
    }
    unsafe{
        let mut vslice=core::slice::from_raw_parts_mut(vpt,size);
        vslice[..cwd.len()].copy_from_slice(cwd.as_bytes());
    }
    buf as isize
}
pub fn sys_chdir(s:*const char)->isize{
    let vstart=VirtAddr::from(s as usize);
    //TODO：不完整的检查
    let vend=VirtAddr::from(s as usize+size_of::<char>());
    if my_thread!().pgtable.check_user_range(vstart, vend,PTEFlags::W)==false{
        return -1;
    }
    let vpt=my_thread!().pgtable.uva_to_kusize(vstart) as *const u8;
    let mut new_path=String::new();
    unsafe{
        for i in 0..4096{ 
            if *vpt.add(i)==0{
                break;
            }
            new_path.push(char::from_u32((*vpt.add(i)) as u32).unwrap());

            if i==4096-1{
                return -1;
            }
        }

    }
    let ret=chdir(&mut my_thread!().cwd,&new_path);
    ret
}

pub fn sys_mkdirat(dirfd:i32,path:*const u8,mode:usize)->isize{

    //TODO:不完整的检查
    let vpt=match user_buf_to_vptr(path as usize,size_of::<char>(),PTEFlags::R){
        None=>{
            return -1;
        }
        Some(p)=>p,
    }as *mut u8 ;
    let mut _path=String::new();
    unsafe{
        for i in 0..4096{
            if *vpt.add(i)==0{
                break;
            }
            _path.push(char::from_u32((*vpt.add(i)) as u32).unwrap());

            if i==4096-1{
                return -1;
            }
        }

    } 
    if (dirfd==-100){
        _path=String::from("/")+my_thread!().cwd.as_str()+_path.as_str();
    }
    if _path.chars().next().unwrap()=='/'{
        return mkdir(FileInner::empty(),&_path);
    }
    let dirfd=dirfd as usize;
    if my_thread!().fd_table.len()<=dirfd{
        return -1;
    }
    let inner=match my_thread!().fd_table[dirfd].clone(){
        None=>{
            return -1;
        }
        Some(inner)=>{
            inner
        }
    };
    mkdir(inner,&_path)
}

pub fn push_inner(inn:FileInner)->usize{
    let mut lk=THREAD_POOL.get_mut().pool[CURRENT_TID.lock()[cpu_id()]].lock();
    for i in 0..lk.as_mut().unwrap().thread.fd_table.len(){
        if lk.as_mut().unwrap().thread.fd_table[i].is_none(){
            lk.as_mut().unwrap().thread.fd_table[i]=Some(inn);
            return i;
        }
    }
    lk.as_mut().unwrap().thread.fd_table.push(Some(inn));
    let ret=lk.as_mut().unwrap().thread.fd_table.len()-1;
    return ret;
}

pub fn sys_openat(dirfd:isize,filename:*const u8,flag:isize,mode:usize)->isize{
    //TODO:flag,mode
    let mut _path=match get_user_string(filename) {
        None=>return -1,
        Some(s)=>s,
    };
    if (dirfd==-100){
        _path=String::from("/")+my_thread!().cwd.as_str()+_path.as_str();
        match  open(FileInner::empty(),&_path,OFlags::from_bits_truncate(mode as u32)){
            Some(inn)=>{
                return push_inner(inn) as isize;
            },
            None=>{
                return -1;
            }
        }
    }
    if dirfd as usize>=my_thread!().fd_table.len(){
        return -1;
    }
    let inner=match my_thread!().fd_table[dirfd as usize].clone(){
        None=>return -1,
        Some(inn)=>inn.clone(),
    };

    println!("4");
    match open(inner, &_path,OFlags::from_bits_truncate(mode as u32)){
        Some(inn)=>{
            return push_inner(inn) as isize;
        },
        None=>{
            return -1;
        }
    }
}

pub fn sys_close(fd:isize)->isize{
    if my_thread!().fd_table.len()>fd as usize{
        if my_thread!().fd_table[fd as usize].is_some(){
            my_thread!().fd_table[fd as usize]=None;
            return 0;
        }
    }
    -1
}

pub fn sys_read(fd:isize,buf:*mut u8,count:usize)->isize{
    let vptr=match user_buf_to_vptr(buf as usize,count,PTEFlags::W){
        None=>return -1,
        Some(vp)=>vp,
    }as *mut u8;
     
    let mut slice=unsafe{slice_from_raw_parts_mut(vptr,count).as_mut().unwrap()};
    if fd  as usize>=my_thread!().fd_table.len(){
        return -1;
    }
    let inn=match my_thread!().fd_table[fd as usize].clone() {
        None=>return -1,
        Some(inner)=>inner,
    };
    fread(inn,slice)   
}

pub fn sys_write(fd:isize,buf:*mut u8,count:usize)->isize{
    let vptr=match user_buf_to_vptr(buf as usize,count,PTEFlags::W){
        None=>return -1,
        Some(vp)=>vp,
    }as *mut u8;
    let slice=unsafe{slice_from_raw_parts(vptr,count).as_ref().unwrap()};
    if fd  as usize>=my_thread!().fd_table.len(){
        return -1;
    }
    let inn=match my_thread!().fd_table[fd as usize].clone() {
        None=>return -1,
        Some(inner)=>inner,
    };
    if inn.std==2{
            unsafe{
                for i in 0..count{
                    print!("{}",*(vptr.add(i)) as char);
                }
            }
            return count  as isize;
    }
    let ret=fwrite(inn,slice);
    ret
}
