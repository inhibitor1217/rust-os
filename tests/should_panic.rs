#![no_std]
#![no_main]

use rust_os::qemu;
use rust_os::serial_print;
use rust_os::serial_println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    qemu::exit(qemu::ExitCode::Failed)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_println!("[ok]");
    qemu::exit(qemu::ExitCode::Success)
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
