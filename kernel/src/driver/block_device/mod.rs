pub mod virtio;
use core::any::Any;
use crate::board::*;
use alloc::{sync::Arc, boxed::Box};


pub trait BlockDevice : Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
}


lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<BlockDeviceImpl> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    println!("block_device_test start");
    let block_device =  BLOCK_DEVICE.clone();
    let mut write_buffer = [0xffu8; 512]; 
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &mut write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}
