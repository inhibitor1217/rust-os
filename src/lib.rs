#![no_std] // disable the standard library
#![cfg_attr(test, no_main)]
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints
#![feature(abi_x86_interrupt)] // enable x86-interrupt ABI
#![feature(custom_test_frameworks)] // enable testing with #[no_std] context
#![test_runner(crate::test_runner)] // define custom test framework runner
#![reexport_test_harness_main = "test_main"] // rename the test entry function to `test_main`

pub mod gdt;
pub mod interrupt;
pub mod qemu;
pub mod serial;
pub mod vga_buffer;

mod test;

pub fn init() {
    gdt::init();
    interrupt::init_idt();
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

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}
