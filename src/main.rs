#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points
#![warn(clippy::all, clippy::pedantic)] // enable clippy lints

mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // this function is the entrypoint, since the linker looks for a function
    // named `_start` by default
    
    use core::fmt::Write;

    write!(vga_buffer::WRITER.lock(), "Hello, world!").unwrap();

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
