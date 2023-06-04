use core::ptr::NonNull;

use alloc::vec::Vec;
use riscv::register::satp;
use virtio_drivers::*;

use spin::Mutex;

use crate::arch::cpu_id;
use crate::memory::address::{pa_usize_to_va_usize};
use crate::memory::page_table::PageTable;
use crate::proc::scheduler::CURRENT_TID;
use crate::proc::thread::THREAD_POOL;
use crate::{memory::{frame::{FrameArea, FrameFlags}, address::{self, pa_to_va_usize, va_usize_to_pa, va_usize_to_pa_usize}, allocator::FRAME_ALLOCATOR, self}, config::PHYS_VIRT_OFFSET};

use core::sync::atomic::{AtomicBool, fence};
use super::BlockDevice;

static vhal:AtomicBool=AtomicBool::new(true);
const VIRTIO0: usize = 0x10001000;

pub struct VirtioBlock(pub Mutex<VirtIOBlk<'static, VirtioHal>>);

impl BlockDevice for VirtioBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0.lock()
            .read_block(block_id, buf)
            .expect("ERROR: read block");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0.lock()
            .write_block(block_id, buf)
            .expect("ERROR: write block")
    }
}


impl VirtioBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        let ret=unsafe {
            Self(Mutex::new(
                VirtIOBlk::new(&mut *((VIRTIO0+PHYS_VIRT_OFFSET) as *mut VirtIOHeader)).unwrap(),
            ))
        };
        ret
    }
}

pub struct VirtioHal;

impl Hal for VirtioHal {
    fn dma_alloc(pages: usize) -> PhysAddr {
        fence(core::sync::atomic::Ordering::SeqCst);
        let ppn=FRAME_ALLOCATOR.lock().alloc(pages).unwrap();
        let ret=usize::from(address::PhysAddr::from(ppn));
        ret
    }
    
    fn dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
        fence(core::sync::atomic::Ordering::SeqCst);
        let ppn=address::PhysAddr::from(paddr).floor_ppn();
        FRAME_ALLOCATOR.lock().dealloc(ppn,pages);
        0
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        fence(core::sync::atomic::Ordering::SeqCst);
        address::pa_usize_to_va_usize(paddr)
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        fence(core::sync::atomic::Ordering::SeqCst);
        let mut pgtable=PageTable::new_by_ppn(satp::read().ppn().into());
        let pa=pgtable.find_va_pa(address::VirtAddr::from(vaddr));
        pa.unwrap().into()
    }
}

//以下为废弃代码，原先为virtio_driver 0.4.0编写,
//但存在未知原因bug，致使死等块设备响应：add_notify_wait_pop函数中死循环
//while !self.can_pop(){
//    spin_loop();
//}

//pub struct VirtioBlock(Mutex<VirtIOBlk<VirtioHal,MmioTransport>>);
/*
impl VirtioBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        println!("VirtIO0 {:#x}",pa_usize_to_va_usize(VIRTIO0));
        let header = NonNull::new(pa_usize_to_va_usize(VIRTIO0) as *mut VirtIOHeader).unwrap();
        let mut transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert!(transport.device_type()==DeviceType::Block);
        println!("max:{}",transport.max_queue_size());
        println!("Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version());
        println!("2");
        //transport.set_status(DeviceStatus::all());
        //println!("status:{:?}",transport.get_status());
        let blk = match unsafe { MmioTransport::new(header) } {
            Err(e) => {
                panic!("Error creating VirtIO MMIO transport: {}", e)
            }
            Ok(transport) => VirtIOBlk::<VirtioHal, MmioTransport>::new(transport)
                .expect("failed to create blk driver"),
        };

        Self(Mutex::new(blk))


    }
}
*/

/*
unsafe impl Hal for VirtioHal {
    #[no_mangle]
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (PhysAddr, core::ptr::NonNull<u8>) {
        let ppn=FRAME_ALLOCATOR.lock().alloc(pages).unwrap();
        //FrameArea::clear_pages(ppn,pages);
        let pa=address::PhysAddr::from(ppn);
        assert!(pa.page_offset()==0);
        let ptr=NonNull::<u8>::new(pa_to_va_usize(pa) as *mut u8).unwrap();
        println!("dma_alloc pa:{:#x} ptr:{:?} pages:{}",usize::from(pa),ptr,pages);
        (usize::from(pa),ptr)
    }

    #[no_mangle]
    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let ppn=address::PhysAddr::from(paddr).floor_ppn();
        FRAME_ALLOCATOR.lock().dealloc(ppn,pages);
        println!("dma_dealloc");
        0
    }
    #[no_mangle]
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        println!("mmio");
        //let pa=address::PhysAddr::from(paddr);
        //let ptr=NonNull::<u8>::new(pa_to_va_usize(pa) as *mut u8).unwrap();
        //ptr
        NonNull::new(paddr as _).unwrap()
    }
    #[no_mangle]
    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        
        let va=buffer.as_ptr() as *mut u8 as usize;
        let current_tid=CURRENT_TID.lock()[cpu_id()];
        let pa=THREAD_POOL.get_mut().pool[current_tid].lock().as_mut().unwrap().thread.pgtable.find_va_pa(VirtAddr::from(va));
        println!("share:buffer_va{:#x},pa:{:#x}",va,usize::from(pa));
        //va_usize_to_pa_usize(va)
        pa.into()
    }
    #[no_mangle]
    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {
        //nothing
        println!("unshare");
    }
}
*/


