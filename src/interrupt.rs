use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame}, instructions};

use crate::{gdt, println, print};

const PIC_1_OFFSET: u8 = 0x20; // Primary Interrupt Controller: Interrupt vectors from 0x20 to 0x27
const PIC_2_OFFSET: u8 = 0x28; // Secondary Interrupt Controller: Interrupt vectors from 0x28 to 0x2f

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET, // Line 0 of Primary Interrupt Controller
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n${:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

pub fn init_pic() {
    unsafe { PICS.lock().initialize(); }
}

pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
