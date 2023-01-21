use core::alloc::GlobalAlloc;

use super::{Locked, align_up};

pub struct Allocator {
    heap_start: u64,
    heap_end: u64,
    next: u64,
    allocations: usize,
}

impl Allocator {
    /// Creates a new empty bump allocator.
    #[must_use]
    pub const fn new() -> Self {
        Allocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// # Safety
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: u64, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size as u64;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size() as u64) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            core::ptr::null_mut() // OOM
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
