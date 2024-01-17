// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use std::ptr::NonNull;

use pneuma::thread::Context;

use crate::thread::ReprContext;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
std::arch::global_asm!(include_str!("asm/aarch64-macos.s"));

#[allow(improper_ctypes)]
extern "C" {
    pub(crate) fn switch_context(store: NonNull<ReprContext>, next: NonNull<ReprContext>);
    pub(crate) fn load_context(cx: NonNull<ReprContext>) -> !;
    pub(crate) fn start_coroutine(next: Context);
}
