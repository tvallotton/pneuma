use std::mem::zeroed;

#[repr(C)]
pub struct Registers {
    sp: u64,
    arg: u64,
    frame: u64,
    link: u64,
    general: [u64; 60],
}

impl Registers {
    fn zeroed() -> Self {
        unsafe { zeroed() }
    }
}
