use core::fmt::Formatter;
use bit_field::BitField;
use bitflags::bitflags;
use crate::virtual_memory::frame::Frame;

const ENTRY_COUNT: usize = 512;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry(u64);

bitflags! {
    #[derive(Debug)]
    pub struct PageEntryFlags: u64 {
        const PRESENT          = 1 << 0;
        const WRITABLE         = 1 << 1;
        const USER_ACCESSIBLE  = 1 << 2;
        const WRITE_THRU       = 1 << 3;
        const NO_CACHE         = 1 << 4;
        const ACCESSED         = 1 << 5;
        const DIRTY            = 1 << 6;
        const HUGE_PAGE        = 1 << 7;
        const GLOBAL           = 1 << 8;
        const _RESERVED_9_3    = 0b111 << 9;
        const _RESERVED_52_11  = (1 << 11 - 1) << 52;
        const NO_EXECUTE       = 1 << 63;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    FrameNotPresent,
    HugeFrame
}

impl PageTableEntry {

    #[inline]
    pub const fn new() -> Self {
        PageTableEntry(0)
    }

    #[inline]
    pub fn addr(&self) -> u64 {
        self.0 & 0x000f_ffff_ffff_f000
    }

    #[inline]
    pub fn set_addr(&mut self, addr: u64) -> &mut Self {
        let mask = (1 << 40) - 1;
        if addr & mask != addr {
            panic!("The physical address exceeds 52 bits!");
        }
        // TODO: assert addr alignment
        // assert!(addr.is_align)
        self.0.set_bits(12..=51, addr);
        self
    }

    #[inline]
    pub fn flags(&self) -> PageEntryFlags {
        PageEntryFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: PageEntryFlags) -> &mut Self {
        self.0 = self.addr() | flags.bits();
        self
    }

    #[inline]
    pub fn frame(&self) -> Result<Frame, FrameError> {
        if !self.flags().contains(PageEntryFlags::PRESENT) {
            Err(FrameError::FrameNotPresent)
        } else if self.flags().contains(PageEntryFlags::HUGE_PAGE) {
            Err(FrameError::HugeFrame)
        } else {
            Ok(Frame::from_addr(self.addr()))
        }
    }

    #[inline]
    pub fn set_frame(&mut self, frame: Frame, flags: PageEntryFlags) {
        assert!(!flags.contains(PageEntryFlags::HUGE_PAGE));
        self.set_addr(frame.addr()).set_flags(flags);
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let result = f.debug_struct("PageTableEntry")
            .field("physical_addr", &format_args!("{:#x}", self.addr()))
            .field("flags", &self.flags())
            .finish();
        result
    }
}


#[derive(Debug)]
#[repr(C)]
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT]
}

impl PageTable {
    #[inline]
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); ENTRY_COUNT],
        }
    }
}