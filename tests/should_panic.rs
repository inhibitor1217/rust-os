#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use rust_os::qemu;
use rust_os::serial_println;
use rust_os::test;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    #[allow(clippy::empty_loop)]
    loop {}
}

pub fn test_runner(tests: &[&dyn test::Testable]) {
    // running multiple test cases are useless for panic tests.
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
        serial_println!("[test did not panic]");
        qemu::exit(qemu::ExitCode::Failed);
    }

    // if no test cases are defined, simply exit.
    qemu::exit(qemu::ExitCode::Success);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_println!("[panicked]");
    qemu::exit(qemu::ExitCode::Success)
}

#[test_case]
fn should_fail() {
    assert_eq!(0, 1);
}
