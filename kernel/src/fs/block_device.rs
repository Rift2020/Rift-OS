use alloc::boxed::Box;
use alloc::vec::*;
use alloc::sync::{Arc, self};

use crate::driver::block_device::virtio::*;
use crate::driver::block_device::*;
use crate::board::BlockDeviceImpl;

#[derive(Clone,Copy,Debug)]
pub struct  BlockDeviceForFS();

unsafe impl Sync for BlockDeviceForFS {
    //什么都没有
}
unsafe impl Send for BlockDeviceForFS {
    //什么都没有
}

impl BlockDeviceForFS {
    pub fn global()->Self{
        Self()
    }
}

impl block_device::BlockDevice for BlockDeviceForFS  {
    type Error=isize;
   fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
       //由于fat32可能会给非512整数倍的数值，所以先用vec读再转过去
       //有待改进
//       assert!(buf.len()>=512);
        if buf.len()<512{
            println!("read {} {}",buf.len(),number_of_blocks);
        }
       let mut buffer:Vec<u8>=Vec::new();
       buffer.resize(512*number_of_blocks,0);
        let mut ind=0;
        for i in address/512..address/512+number_of_blocks{
            unsafe{
                    BLOCK_DEVICE.clone()
                    .read_block(i, &mut buffer.as_mut_slice()[512*ind..512*(ind+1)])
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
                    BLOCK_DEVICE.clone()
                    .write_block(i, & buf[0..512])
            }
        }
       Ok(())
   }
}




