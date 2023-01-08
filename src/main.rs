#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // this function is the entrypoint, since the linker looks for a function
    // named `_start` by default
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
