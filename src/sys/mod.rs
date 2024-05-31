use std::ptr::NonNull;

use pneuma::uthread::{Context, ReprContext};

pub(crate) mod signal_stack;
pub(crate) mod stack;
pub(crate) mod statx;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
std::arch::global_asm!(include_str!("asm/aarch64-macos.s"));

#[allow(improper_ctypes)]
extern "C" {
    /// Safety:
    /// Caller must ensure that these threads are not currently being
    /// scheduled by another worker.
    pub(crate) fn switch_context(store: Context, next: Context) -> [Context; 2];
    #[allow(dead_code)]
    pub(crate) fn start_coroutine(next: NonNull<ReprContext>);
}

#[macro_export]
macro_rules! syscall {
    ($fun:ident$(, $($arg:expr),*)? $(,)?) => {
        unsafe {
            let res = libc::$fun($($($arg),*)?);
            if let -1 = res as _  {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(res)
            }
        }
    }
}
