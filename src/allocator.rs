use core::{alloc::GlobalAlloc, ptr::null_mut};

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub mod bump;
pub mod linked_list;

pub struct Stub;

unsafe impl GlobalAlloc for Stub {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub const KERNEL_HEAP_START: u64 = 0x_4444_4444_0000; // An arbitrary value
pub const KERNEL_HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: Locked<linked_list::Allocator> = Locked::new(linked_list::Allocator::new());

/// Initializes the kernel heap memory.
/// 
/// # Errors
/// The initialization might fail if the frame allocator fails
/// to allocate enough physical memory frames for kernel heap memory
/// or the page tables for virtual memory mappings.
pub fn init_kernel_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(KERNEL_HEAP_START);
        let heap_end = heap_start + KERNEL_HEAP_SIZE - 1u64;
        Page::range_inclusive(
            Page::containing_address(heap_start),
            Page::containing_address(heap_end),
        )
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(KERNEL_HEAP_START, KERNEL_HEAP_SIZE);
    }

    Ok(())
}

/// Align the given address `addr` upwards to alignment `align`.
fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}
