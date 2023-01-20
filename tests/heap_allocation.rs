#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use alloc::{boxed::Box, vec};
use bootloader::{entry_point, BootInfo};
use rust_os::{
    allocator::{self, KERNEL_HEAP_SIZE}, init,
    memory::{self, BootInfoFrameAllocator},
    test_panic_handler,
};
use x86_64::VirtAddr;

extern crate alloc;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_kernel_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[test_case]
fn simple_box() {
    let fourty_one = Box::new(41);
    let thirteen = Box::new(13);

    assert_eq!(*fourty_one, 41);
    assert_eq!(*thirteen, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = vec![];
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..KERNEL_HEAP_SIZE {
        // Might run out of memory here, allocator should re-use freed memory
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
