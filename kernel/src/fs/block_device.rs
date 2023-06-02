use alloc::boxed::Box;
use alloc::vec::*;
use alloc::sync::{Arc, self};

use crate::driver::block_device::virtio::*;
use block_device::BlockDevice;
use crate::driver::block_device::BLOCK_DEVICE;
use crate::board::BlockDeviceImpl;

#[derive(Clone,Copy,Debug)]
pub struct  BlockDeviceForFS(*mut Arc<BlockDeviceImpl>);

unsafe impl Sync for BlockDeviceForFS {
    //什么都没有
}
unsafe impl Send for BlockDeviceForFS {
    //什么都没有
}

impl BlockDeviceForFS {
    pub fn global()->Self{
        Self(& *BLOCK_DEVICE as *const Arc<BlockDeviceImpl> as *mut Arc<BlockDeviceImpl>)
    }
}

impl BlockDevice for BlockDeviceForFS  {
    type Error=isize;
   fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
       assert!(buf.len()>=512);
       //由于fat32可能会给非512整数倍的数值，所以先用vec读再转过去
       //有待改进
       let mut buffer:Vec<u8>=Vec::new();
       buffer.resize(512*number_of_blocks,0);
        let mut ind=0;
        for i in address/512..address/512+number_of_blocks{
            unsafe{
                self.0.as_mut().unwrap().0.lock()
                    .read_block(i, &mut buffer.as_mut_slice()[512*ind..512*(ind+1)])
                    .expect("ERROR: read block");
            }
            ind=ind+1;
        }
        buf.copy_from_slice(&buffer[..buf.len()]);
        Ok(())
   }
   fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
       assert!(buf.len()>=512);
        for i in address/512..address/512+number_of_blocks{
            unsafe{
                self.0.as_mut().unwrap().0.lock()
                    .write_block(i, & buf[0..512])
                    .expect("ERROR: write_block");
            }
        }
       Ok(())
   }
}




