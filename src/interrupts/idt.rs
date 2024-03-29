use x86_64::instructions::segmentation;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::{PrivilegeLevel, VirtAddr};

use bit_field::BitField;
use crate::interrupts::hardware::InterruptIndex;

type HandlerWrapper = extern "C" fn() -> !;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
pub enum CpuExceptionIndex {
    DivisionError          = 0x0,
    Debug                  = 0x1,
    NonMaskableInterrupt   = 0x2,
    Breakpoint             = 0x3,
    Overflow               = 0x4,
    BoundRangeExceeded     = 0x5,
    InvalidOpcode          = 0x6,
    DeviceNotAvailable     = 0x7,
    DoubleFault            = 0x8,
    CoprocessorSegOverrun  = 0x9,
    InvalidTSS             = 0xa,
    SegmentNotPresent      = 0xb,
    StackSegmentFault      = 0xc,
    GeneralProtectionFault = 0xd,
    PageFault              = 0xe,
    Reserved               = 0xf,
}

impl CpuExceptionIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
pub enum IdtIndex {
    CpuException(CpuExceptionIndex),
    Interrupt(InterruptIndex),
}

impl IdtIndex {
    pub fn as_u8(self) -> u8 {
        match self {
            IdtIndex::CpuException(cpu_exception_index) => cpu_exception_index.as_u8(),
            IdtIndex::Interrupt(interrupt_index) => interrupt_index.as_u8(),
        }
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }

}

#[derive(Debug)]
pub struct Idt([Entry; 64]);

impl Idt {
    pub fn new() -> Self {
        Idt([Entry::missing(); 64])
    }

    pub fn set_handler(&mut self, entry: IdtIndex, handler_func: HandlerWrapper) -> &mut EntryOptions {
        self.0[entry.as_usize()] = Entry::new(segmentation::CS::get_reg(), handler_func);
        unsafe {
            let raw_ptr = core::ptr::addr_of_mut!(self.0[entry.as_usize()].options);
            &mut *raw_ptr
        }
    }

    pub fn load(&'static self) {
        use x86_64:: instructions::tables::{DescriptorTablePointer, lidt};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {

    fn missing() -> Self {
        Entry {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
    fn new(gdt_selector: SegmentSelector, handler: HandlerWrapper) -> Self {
        let pointer = handler as u64;
       Entry {
            gdt_selector,
            pointer_low: (pointer & 0xffff) as u16,
            pointer_middle: ((pointer >> 16) & 0xffff) as u16,
            pointer_high: ((pointer >> 32) & 0xffffffff) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

#[allow(dead_code)]
impl EntryOptions {

    fn minimal() -> Self {
        let mut option = 0;
        // Be cautious about the ordering of the bits
        //    12 11 10  9
        // 0b  0  1  1  1
        option.set_bits(9..12, 0b111);
        EntryOptions(option)
    }
    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disabled: bool) -> &mut Self {
        self.0.set_bit(8, !disabled);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_bits(13..15, dpl & 0b11);
        self
    }

    /* trick here.
     in GDT, the array indices start at 0, [0, 6]
        However, the IDT options should be [1, 7]  0 is reserved for default
     */
    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, (index + 1) & 0b111);
        self
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test_case]
    pub fn test_idt_index_value() {

        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::DivisionError).as_u8(), 0x0);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::Debug).as_u8(), 0x1);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::NonMaskableInterrupt).as_u8(), 0x2);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::Breakpoint).as_u8(), 0x3);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::Overflow).as_u8(), 0x4);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::BoundRangeExceeded).as_u8(), 0x5);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::InvalidOpcode).as_u8(), 0x6);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::DeviceNotAvailable).as_u8(), 0x7);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::DoubleFault).as_u8(), 0x8);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::CoprocessorSegOverrun).as_u8(), 0x9);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::InvalidTSS).as_u8(), 0xa);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::SegmentNotPresent).as_u8(), 0xb);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::StackSegmentFault).as_u8(), 0xc);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::GeneralProtectionFault).as_u8(), 0xd);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::PageFault).as_u8(), 0xe);
        assert_eq!(IdtIndex::CpuException(CpuExceptionIndex::Reserved).as_u8(), 0xf);

        // interrupts
        assert_eq!(IdtIndex::Interrupt(InterruptIndex::Timer).as_u8(), 32);
    }

}