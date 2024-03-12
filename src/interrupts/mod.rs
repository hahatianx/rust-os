
pub mod idt;
pub mod hardware;
mod page_fault;

use core::arch::asm;
use lazy_static::lazy_static;

use crate::interrupts::idt::{Idt, IdtIndex};
use crate::interrupts::page_fault::PageFaultErrorCode;
use crate::gdt;
use crate::println;

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}


#[macro_export]
macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    // save scratch registers
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push rsi",
                    "push rdi",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",

                    "mov rdi, rsp",
                    "add rdi, 72",
                    "call {func}",

                    // restore scratch registers after func call
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdi",
                    "pop rsi",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",

                    "iretq",
                    func = sym $name,
                    options(noreturn)
                );
            }
        }
        wrapper
    }};
}

#[macro_export]
macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    // save scratch registers
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push rsi",
                    "push rdi",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",

                    "mov rsi, [rsp + 8 * 9]",
                    "mov rdi, rsp",
                    "add rdi, 8 * 10",

                    // this is because now we have 80 addtional bytes on top of existing stack
                    // It becomes not aligned again
                    "sub rsp, 8",

                    "call {func}",

                    "add rsp, 8",

                    // restore scratch registers after func call
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdi",
                    "pop rsi",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",

                    // remove error code from the stack.
                    // after that, rsp points to stack_frame which causes the error
                    "add rsp, 8",
                    "iretq",
                    func = sym $name,
                    options(noreturn)
                );
            }
        }
        wrapper
    }};
}

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.set_handler(IdtIndex::DoubleFault, handler_with_error_code!(double_fault_handler))
            .set_stack_index(gdt::ISTIndex::DoubleFaultISTIndex as u16);
        idt.set_handler(IdtIndex::DivisionError, handler!(divide_by_zero_exception));
        idt.set_handler(IdtIndex::Breakpoint, handler!(breakpoint_exception));
        idt.set_handler(IdtIndex::InvalidOpcode, handler!(invalid_opcode_handler));
        idt.set_handler(IdtIndex::PageFault, handler_with_error_code!(page_fault_handler));
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "C" fn breakpoint_exception(stack_frame: &ExceptionStackFrame) {
    println!("\nBREAKPOINT\n{:#?}", stack_frame);
}

extern "C" fn divide_by_zero_exception(stack_frame: &ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!("\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame);
}

extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    use x86_64::registers::control;
    println!("\nEXCEPTION: PAGE FAULT while accessing {:#x}\
        \nerror code: {:?}\n{:#?}",
        control::Cr2::read().unwrap(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame);
}

extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[cfg(test)]
mod test {
    #[test_case]
    fn test_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }

    // The following two tests are commented out on purpose
    // The instruction fails the CPU cannot get bypassed, causing the kernel into a dead loop

    #[test_case]
    fn test_divide_by_zero_exception() {
        /*
        use core::arch::asm;
        unsafe {
            asm!(
                "mov rdx, 0",
                "div rdx",
                out("rdx") _,
                out("rax") _,
            );
        }
         */
    }

    #[test_case]
    #[allow(deref_nullptr)]
    fn test_page_fault_exception() {
        // unsafe { *(0x0 as *mut u64) = 1 };
    }
}