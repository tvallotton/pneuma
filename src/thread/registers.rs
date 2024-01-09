use std::mem::zeroed;

// pub type Registers = [u64; 19];

#[repr(C)]
pub struct Registers {
    pub sp: u64,
    pub fun: u64,
    pub arg: u64,
    pub frame: u64,
    pub link: u64,
    pub general: [u64; 59],
}

impl Registers {
    fn zeroed() -> Self {
        unsafe { zeroed() }
    }
}
