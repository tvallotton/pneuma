use std::mem::zeroed;

#[cfg(target_arch = "aarch64")]
pub type Registers = [u64; 19];
