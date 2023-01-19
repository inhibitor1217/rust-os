#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints
#![feature(custom_test_frameworks)] // enable testing with #[no_std] context
#![test_runner(rust_os::test_runner)] // define custom test framework runner
#![reexport_test_harness_main = "test_main"] // rename the test entry function to `test_main`

use rust_os::memory::{self, BootInfoFrameAllocator};
use x86_64::{structures::paging::Page, VirtAddr};

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    rust_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    rust_os::halt();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_os::println!("{info}");

    rust_os::halt();
}

/// Panic handler in test mode.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}
