use x86_64::instructions::segmentation;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::{PrivilegeLevel, VirtAddr};

use bit_field::BitField;

type HandlerWrapper = extern "C" fn() -> !;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum IdtIndex {
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

#[derive(Debug)]
pub struct Idt([Entry; 16]);

impl Idt {
    pub fn new() -> Self {
        Idt([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: IdtIndex, handler_func: HandlerWrapper) -> &mut EntryOptions {
        self.0[entry as usize] = Entry::new(segmentation::CS::get_reg(), handler_func);
        unsafe {
            let raw_ptr = core::ptr::addr_of_mut!(self.0[entry as usize].options);
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

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index & 0b111);
        self
    }
}