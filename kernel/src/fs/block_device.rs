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
       println!("address:{}",address);
        for i in address/512..address/512+number_of_blocks{
            unsafe{
                self.0.as_mut().unwrap().0.lock()
                    .read_block(i, buf)
                    .expect("ERROR: read block");
            }
        }
        Ok(())
   }
   fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error> {
        for i in address/512..address/512+number_of_blocks{
            unsafe{
                self.0.as_mut().unwrap().0.lock()
                    .write_block(i, buf)
                    .expect("ERROR: write_block");
            }
        }
       Ok(())
   }
}




