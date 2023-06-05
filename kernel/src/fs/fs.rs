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

#[derive(Clone,Debug)]
pub struct FileInner{
   pub dir:Option<Dir<'static,BlockDeviceForFS>>,
   pub file:Option<File<'static,BlockDeviceForFS>>,
   pub std:usize,
   pub path:String,
   pub flag:u8,
}

impl FileInner {
    pub fn empty()->Self{
        FileInner { dir: None, file: None,std:0 ,path: String::new(), flag: 0 }
    }
}

pub fn chdir_vec<'a>(cwd_v:&mut Vec<&'a str>,path_vec:Vec<&'a str>)->isize{
    let mut cwd_vec:Vec<&str>=cwd_v.clone();
    if path_vec.len()==0{
        return -1;
    }
    //根目录开始
    if path_vec[0]==""{
        cwd_vec=path_vec.clone();
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
    if cwd_vec[0]==""{
        cwd_vec.remove(0);
    }
    let ret=FILE_SYSTEM.check_path_vec(&cwd_vec[..]);
    if ret==0{
        *cwd_v=cwd_vec.clone();
    }

    ret
}

pub fn chdir(cwd:&mut String,path:&String)->isize{
    let mut cwd_vec:Vec<&str>=cwd.split('/').collect();
    let path_vec:Vec<&str>=path.split('/').collect();
    let ret=chdir_vec(&mut cwd_vec,path_vec);
    if ret==0{
        *cwd=cwd_vec.join("/");
    }
    ret
}

pub fn walk(dir:&mut Dir<'static,BlockDeviceForFS>,path_slice:&[&str])->isize{
    let mut d=dir.clone();
    for i in 0..path_slice.len(){
        if path_slice[i]=="."{
            continue;
        }
        if path_slice[i]==".."{
            panic!("walk:TODO");
        }
        match d.cd(path_slice[i])  {
            Ok(new_dir)=>{
                d=new_dir;
            }
            Err(e)=>{
                return -1;
            }
        } 
    }
    *dir=d;
    0

}

pub fn mkdir(dir:FileInner,path:&String)->isize{
    let path_vec:Vec<&str>=path.split('/').collect();
    let mut mdir=FILE_SYSTEM.root_dir();
    let mut walk_ret=0;
    if path_vec[0]==""{
        walk_ret=walk(&mut mdir,&path_vec[1..path_vec.len()-1]);
    }
    else{
        if dir.dir.is_none(){
            return -1;
        }
        mdir=dir.dir.unwrap();
        walk_ret=walk(&mut mdir,&path_vec[..path_vec.len()-1]);
    }
    if walk_ret==0&&mdir.create_dir(&path_vec[path_vec.len()-1]).is_ok(){
        return 0;
    }
    else{
        return -1;
    }
}
pub fn open(dir:FileInner,path:&String)->Option<FileInner>{
    let path_vec:Vec<&str>=path.split('/').collect();
    let mut mdir=FILE_SYSTEM.root_dir();
    let mut walk_ret=0;
    if path_vec[0]==""{
        walk_ret=walk(&mut mdir,&path_vec[1..path_vec.len()-1]);
    }
    else{
        if dir.dir.is_none(){
            return None;
        }
        mdir=dir.dir.unwrap();
        walk_ret=walk(&mut mdir,&path_vec[..path_vec.len()-1]);
    }
    if walk_ret==0{
        match mdir.open_file(path_vec[path_vec.len()-1]) {
            Ok(file_)=>{
                return Some(FileInner { dir: None, file: Some(file_),std:0, path: path.clone(), flag: 0 });
            }
            Err(e)=>{
                match mdir.cd(path_vec[path_vec.len()-1]) {
                        Ok(dir)=>{
                            return Some(FileInner { dir: Some(dir), file: None,std:0, path: path.clone(), flag: 0 });
                        },
                        Err(e)=>{
                            return None;
                        
                        }
                }
            }
        } 

    }
    return None;
}
