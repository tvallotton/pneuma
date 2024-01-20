// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use crate::thread::ReprContext;
use pneuma::thread::Context;
use std::ptr::NonNull;

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

#[macro_export]
macro_rules! syscall {
    ($fun:ident$(, $($arg:expr),*)? $(,)?) => {
        unsafe {
            let res = libc::$fun($($($arg),*)?);
            if let -1 = res  {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }
    }
}
