#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints
#![feature(custom_test_frameworks)] // enable testing with #[no_std] context
#![test_runner(rust_os::test_runner)] // define custom test framework runner
#![reexport_test_harness_main = "test_main"] // rename the test entry function to `test_main`

use rust_os::{memory::{translate_addr}, println};
use x86_64::VirtAddr;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    rust_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, physical_memory_offset) };
        println!("{virt:?} -> {phys:?}");
    }

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
