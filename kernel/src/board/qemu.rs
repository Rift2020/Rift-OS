pub type BlockDeviceImpl = crate::driver::block_device::virtio::VirtioBlock;
pub const MMIO: &[(usize, usize)] = &[(0x10001000, 0x1000)];
