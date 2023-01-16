use x86_64::{registers::control::Cr3, structures::paging::{PageTable, page_table::FrameError}, PhysAddr, VirtAddr};

/// Returns a mutable reference to the active level 4 page table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to the virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be called only once
/// to avoid aliasing `&mut`
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Translates the given virtual address to the mapped physical address,
/// or `None` if the address is not mapped.
/// 
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to the virtual memory at the passed
/// `physical_memory_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_impl(addr, physical_memory_offset)
}

fn translate_addr_impl(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;

    for &index in &table_indices {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages are not supported"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
