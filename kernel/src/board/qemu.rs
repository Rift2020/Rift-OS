pub type BlockDeviceImpl = crate::driver::block_device::virtio::VirtioBlock;
pub const MMIO: &[(usize, usize)] = &[(0x10001000, 0x1000)];

pub const CPU_FREQ :usize=1_500_000_000;//好像没有方法来设置qemu
                                        //cpu的频率，不同的机器在该值上将会有较大的差异
