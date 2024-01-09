// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use std::ptr::NonNull;

use pneuma::thread::Thread;

use pneuma::thread::RcContext;

use crate::thread::Context;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
std::arch::global_asm!(include_str!("asm/aarch64-macos.s"));

// pub(crate) use _switch_context as switch_context;
// pub(crate) use _switch_no_save as switch_no_save;
extern "C" {
    pub(crate) fn switch_context(store: NonNull<Context>, next: NonNull<Context>);
    pub(crate) fn start_coroutine(next: RcContext);
}
