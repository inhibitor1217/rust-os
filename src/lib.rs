#![no_std] // disable the standard library
#![cfg_attr(test, no_main)]
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints
#![feature(abi_x86_interrupt)] // enable x86-interrupt ABI
#![feature(alloc_error_handler)] // enable handling allocation failure
#![feature(const_mut_refs)] // enable allowing mutable references in const fn
#![feature(custom_test_frameworks)] // enable testing with #[no_std] context
#![test_runner(crate::test_runner)] // define custom test framework runner
#![reexport_test_harness_main = "test_main"] // rename the test entry function to `test_main`

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod vga_buffer;

mod test;

pub fn init() {
    gdt::init();
    interrupt::init_idt();
    interrupt::init_pic();
    interrupt::enable_interrupts();
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_runner(tests: &[&dyn test::Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit(qemu::ExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {info}\n");
    qemu::exit(qemu::ExitCode::Failed);
}

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
