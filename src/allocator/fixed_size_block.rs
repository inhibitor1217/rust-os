use core::alloc::GlobalAlloc;

use super::Locked;

struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// The block sizes to use.
/// 
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

const EMPTY: Option<&'static mut ListNode> = None;

pub struct Allocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl Allocator {
    /// Creates an empty Allocator.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    /// 
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. Also, this method must
    /// be called only once.
    pub unsafe fn init(&mut self, heap_start: u64, heap_size: u64) {
        self.fallback_allocator.init(heap_start as *mut u8, heap_size as usize);
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        if let Ok(ptr) = self.fallback_allocator.allocate_first_fit(layout) {
            ptr.as_ptr()
        } else {
            core::ptr::null_mut()
        }
    }
}

unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();
        if let Some(index) = list_index(layout) {
            if let Some(head) = allocator.list_heads[index].take() {
                allocator.list_heads[index] = head.next.take();
                (head as *mut ListNode).cast::<u8>()
            } else {
                let block_size = BLOCK_SIZES[index];
                let block_align = block_size;
                let layout = core::alloc::Layout::from_size_align(block_size, block_align).unwrap();
                allocator.fallback_alloc(layout)
            }
        } else {
            allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        if let Some(index) = list_index(layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };
            assert!(core::mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(core::mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
            let new_node_ptr = ptr.cast::<ListNode>();
            new_node_ptr.write(new_node);
            allocator.list_heads[index] = Some(&mut *new_node_ptr);
        } else {
            let ptr = core::ptr::NonNull::new(ptr).unwrap();
            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}

/// Choose an appropriate block size for given layout.
/// 
/// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: core::alloc::Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}
