

#[derive(Debug)]
#[repr(C)]
pub struct Frame(u64);

impl Frame {

    #[inline]
    pub const fn from_addr(addr: u64) -> Self {
        Frame(addr)
    }

    #[inline]
    pub fn addr(&self) -> u64 {
        self.0
    }

}