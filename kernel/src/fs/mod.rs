pub mod fs;
pub mod block_device;
use fat32::volume::Volume;
use crate::driver::block_device::BLOCK_DEVICE;
use crate::driver::block_device::virtio::VirtioBlock;
use crate::board::BlockDeviceImpl;
use self::block_device::BlockDeviceForFS;
use spin::Mutex;

use alloc::sync::Arc;

lazy_static!{
    pub static ref FILE_SYSTEM:Mutex<Volume<BlockDeviceForFS>>=Mutex::new(Volume::new(BlockDeviceForFS::global()));
}
