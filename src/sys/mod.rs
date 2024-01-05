// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use std::slice::RChunks;

use pneuma::thread::Thread;

use pneuma::thread::RcContext;

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
std::arch::global_asm!(include_str!("asm/aarch64-macos.s"),);

// pub(crate) use _switch_context as switch_context;
// pub(crate) use _switch_no_save as switch_no_save;
extern "C" {
    pub(crate) fn switch_context(
        store: Thread,
        next: Thread,
        f: extern "C" fn(RcContext, RcContext),
    );
    pub(crate) fn switch_no_save(next: RcContext);
    pub(crate) fn start_coroutine(store: RcContext, next: RcContext, x: u64);
}
