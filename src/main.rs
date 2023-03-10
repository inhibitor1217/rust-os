#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints
#![feature(custom_test_frameworks)] // enable testing with #[no_std] context
#![test_runner(rust_os::test_runner)] // define custom test framework runner
#![reexport_test_harness_main = "test_main"] // rename the test entry function to `test_main`

extern crate alloc;

use rust_os::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    task::{executor::Executor, keyboard, Task},
};
use x86_64::VirtAddr;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    rust_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_kernel_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypress()));
    #[cfg(test)]
    executor.spawn(Task::new(async {
        test_main();
    }));

    executor.run();
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
