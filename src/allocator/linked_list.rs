use core::alloc::GlobalAlloc;

use crate::allocator::align_up;

use super::Locked;

struct ListNode {
    size: u64,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: u64) -> Self {
        Self {
            size,
            next: None,
        }
    }

    fn start_addr(&self) -> u64 {
        self as *const Self as u64
    }

    fn end_addr(&self) -> u64 {
        self.start_addr() + self.size
    }
}

pub struct Allocator {
    head: ListNode,
}

impl Allocator {
    /// Creates an empty `Allocator`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    /// 
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. Also, this method must
    /// be called only once.
    pub unsafe fn init(&mut self, heap_start: u64, heap_size: u64) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Adds the given memory region to the front of the list.
    unsafe fn add_free_region(&mut self, addr: u64, size: u64) {
        // ensure that the freed region is capable of holding ListNode
        assert_eq!(align_up(addr, core::mem::align_of::<ListNode>() as u64), addr);
        assert!(size >= core::mem::size_of::<ListNode>() as u64);

        // create a new list node and append to the start of list
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    /// Looks for a free region with the given size and alignment and removes
    /// it from the list.
    /// 
    /// Returns a tuple of the list node and the start address of the allocation.
    fn find_region(&mut self, size: u64, align: u64) -> Option<(&'static mut ListNode, u64)> {
        let mut current = &mut self.head;
        
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                let next = region.next.take();
                let ret = (current.next.take().unwrap(), alloc_start);
                current.next = next;
                return Some(ret);
            }
            current = current.next.as_mut().unwrap();
        }

        None
    }

    /// Try to use the given region for an allocation with given size and alignment.
    /// 
    /// Returns the allocation start address on success.
    fn alloc_from_region(region: &ListNode, size: u64, align: u64) -> Result<u64, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // Region is to small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() as u64 {
            // rest of the region is too small to hold a ListNode
            return Err(());
        }

        Ok(alloc_start)
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is also aligned to storing a `ListNode`.
    fn size_align(layout: core::alloc::Layout) -> (u64, u64) {
        let layout = layout
            .align_to(core::mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();

        let size = layout.size().max(core::mem::size_of::<ListNode>());
        (size as u64, layout.align() as u64)
    }
}

unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = Allocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let (size, _) = Allocator::size_align(layout);
        self.lock().add_free_region(ptr as u64, size);
    }
}
