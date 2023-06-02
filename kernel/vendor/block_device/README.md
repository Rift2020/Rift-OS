# BlockDevice trait
```rust
pub trait BlockDevice {
    const BLOCK_SIZE: u32 = 512;
    
    type Error;
    fn read(&self, buf: &mut [u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error>;
    fn write(&self, buf: &[u8], address: usize, number_of_blocks: usize) -> Result<(), Self::Error>;
}
```