use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    marker::PhantomData,
};
use linked_list_allocator::Heap;
use tock_registers::RegisterLongName;

use crate::{errors::Errcode, sync::RwLock};

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::init();

pub const BOARD_DEFAULT_LOAD_ADDRESS: usize = 0x8_0000;
pub const BASE: usize = 0x3F00_0000;
pub const SYSTIMER_BASE: usize = BASE + 0x0000_3000;
pub const TXP_BASE: usize = BASE + 0x0000_4000;
pub const DMA_BASE: usize = BASE + 0x0000_7000;
pub const INTERRUPT_CTRL_BASE: usize = BASE + 0x0000_B200;
pub const VCHIQ_BASE: usize = BASE + 0x0000_B840;
pub const MAILBOX_BASE: usize = BASE + 0x0000_B880;
pub const WATCHDOG_BASE: usize = BASE + 0x0010_0000;
pub const CPRMAN_BASE: usize = BASE + 0x0010_1000;
pub const RANDOM_BASE: usize = BASE + 0x0010_4000;
pub const GPIO_BASE: usize = BASE + 0x0020_0000;
pub const UART0_BASE: usize = BASE + 0x0020_1000;
pub const MMC0_BASE: usize = BASE + 0x0020_2000;
pub const I2S_BASE: usize = BASE + 0x0020_3000;
pub const SPI0_BASE: usize = BASE + 0x0020_4000;
pub const I2C0_BASE: usize = BASE + 0x0020_5000;
pub const PIXELVALVE0_BASE: usize = BASE + 0x0020_6000;
pub const PIXELVALVE1_BASE: usize = BASE + 0x0020_7000;
pub const DPI_BASE: usize = BASE + 0x0020_8000;
pub const DSI0_BASE: usize = BASE + 0x0020_9000;
pub const PWM_BASE: usize = BASE + 0x0020_C000;
pub const THERMAL_BASE: usize = BASE + 0x0021_2000;
pub const AUX_BASE: usize = BASE + 0x0021_5000;
pub const UART1_BASE: usize = BASE + 0x0021_5040;
pub const SPI1_BASE: usize = BASE + 0x0021_5080;
pub const SPI2_BASE: usize = BASE + 0x0021_50C0;
pub const MMC1_BASE: usize = BASE + 0x0030_0000;
pub const HVS_BASE: usize = BASE + 0x0040_0000;
pub const SMI_BASE: usize = BASE + 0x0060_0000;
pub const DSI1_BASE: usize = BASE + 0x0070_0000;
pub const CSI0_BASE: usize = BASE + 0x0080_0000;
pub const CSI1_BASE: usize = BASE + 0x0080_1000;
pub const I2C1_BASE: usize = BASE + 0x0080_4000;
pub const I2C2_BASE: usize = BASE + 0x0080_5000;
pub const VEC_BASE: usize = BASE + 0x0080_6000;
pub const PIXELVALVE2_BASE: usize = BASE + 0x0080_7000;
pub const HDMI_BASE: usize = BASE + 0x0090_2000;
pub const USB_BASE: usize = BASE + 0x0098_0000;
pub const V3D_BASE: usize = BASE + 0x00C0_0000;

extern "Rust" {
    static __heap_start: UnsafeCell<()>;
    static __heap_end_exclusive: UnsafeCell<()>;
}

pub struct MMIODerefWrapper<T> {
    start_addr: usize,
    phantom: PhantomData<fn() -> T>,
}

impl<T> MMIODerefWrapper<T> {
    /// Create an instance.
    pub const fn new(start_addr: usize) -> Self {
        Self {
            start_addr,
            phantom: PhantomData,
        }
    }
}

impl<T> core::ops::Deref for MMIODerefWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.start_addr as *const _) }
    }
}

impl<T> RegisterLongName for MMIODerefWrapper<T> {}

pub struct HeapAllocator {
    heap: RwLock<Heap>,
}

impl HeapAllocator {
    const fn init() -> HeapAllocator {
        HeapAllocator {
            heap: RwLock::new(Heap::empty()),
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let result = self.heap.write(|a| a.allocate_first_fit(layout).ok());

        match result {
            None => core::ptr::null_mut(),
            Some(allocation) => allocation.as_ptr(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.heap
            .write(|inner| inner.deallocate(core::ptr::NonNull::new_unchecked(ptr), layout));
    }
}

pub(crate) fn init() -> Result<(), Errcode> {
    unsafe {
        let heap_addr = __heap_start.get() as *mut u8;
        let heap_size = (__heap_end_exclusive.get() as usize) - (__heap_start.get() as usize);
        HEAP_ALLOCATOR.heap.write(|h| h.init(heap_addr, heap_size));
    }
    Ok(())
}
