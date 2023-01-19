use core::{alloc::GlobalAlloc, ptr::null_mut};

pub struct Stub;

unsafe impl GlobalAlloc for Stub {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

#[global_allocator]
static ALLOCATOR: Stub = Stub;
