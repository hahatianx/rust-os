#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::arch::asm;
use lazy_static::lazy_static;

use blog_os::{exit_qemu, handler_with_error_code, QemuExitCode, serial_print, serial_println};
use blog_os::interrupts::{ExceptionStackFrame};
use blog_os::interrupts::idt::{IdtIndex, Idt};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    blog_os::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info);
}

lazy_static! {
    static ref TEST_IDT: Idt = {
        let mut idt = Idt::new();
        idt.set_handler(IdtIndex::DoubleFault, handler_with_error_code!(test_double_fault_handler))
            .set_stack_index(blog_os::gdt::ISTIndex::DoubleFaultISTIndex as u16);
        idt
    };
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "C" fn test_double_fault_handler(_stack_frame: &ExceptionStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}