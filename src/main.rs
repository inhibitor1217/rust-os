#![no_std] // disable the standard library
#![no_main] // disable all Rust-level entry points

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // this function is the entrypoint, since the linker looks for a function
    // named `_start` by default
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in b"Hello, World!".iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
