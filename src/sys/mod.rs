// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use crate::task::Context;

// #[cfg(all(target_family = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

extern "C" {
    pub(crate) fn switch_context(store: *mut Context, next: *const Context);
}
