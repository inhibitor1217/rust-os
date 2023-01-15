#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)] // enable x86-interrupt ABI

use lazy_static::lazy_static;
use rust_os::{gdt, qemu, serial_print, serial_println, test_panic_handler};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    qemu::exit(qemu::ExitCode::Success);
}

fn init_test_idt() {
    TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // push return address to the stack
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}
