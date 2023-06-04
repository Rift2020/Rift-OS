use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use fat32::dir::Dir;
use fat32::entry::*;
use alloc::vec::*;
use alloc::collections::VecDeque;
use fat32::file::File;
use fat32::volume::Volume;
use riscv::_export::critical_section::Mutex;
use super::FILE_SYSTEM;
use super::BlockDeviceForFS;

pub struct File_(Arc<Mutex<FileInner>>);

pub struct FileInner{
   dir:Option<Dir<'static,BlockDeviceForFS>>,
   file:Option<File<'static,BlockDeviceForFS>>,
    flag:u8,
}






pub fn chdir(cwd:&mut String,path:&String)->isize{
    let mut cwd_vec:Vec<&str>=cwd.split('/').collect();
    let path_vec:Vec<&str>=path.split('/').collect();
    let mut new_cwd=String::new();
    if path_vec.len()==0{
        return -1;
    }
    //根目录开始
    if path_vec[0]==""{
        cwd_vec=path_vec;
    }
    //相对路径
    else{
        for i in path_vec{
            match i {
                ""=>{
                    return -1;
                }
                "."=>{
                    ;
                }
                ".."=>{
                    cwd_vec.pop();
                }
                _ =>{
                    cwd_vec.push(i);
                }
            }
        }
    }
    let ret=FILE_SYSTEM.lock().check_path_vec(&cwd_vec);
    if ret==0{
        *cwd=cwd_vec.join("/");
    }
    ret
}
