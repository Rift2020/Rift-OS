pub mod block_device;
use fat32::volume::Volume;
use crate::driver::block_device::BLOCK_DEVICE;
use crate::driver::block_device::virtio::VirtioBlock;
use crate::board::BlockDeviceImpl;
use self::block_device::BlockDeviceForFS;

use alloc::sync::Arc;
lazy_static!{
    pub static ref FILE_SYSTEM:Volume<BlockDeviceForFS>=Volume::new(BlockDeviceForFS::global());
}
