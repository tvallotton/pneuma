// #[cfg(target_arch = "aarch64")]
// pub use aarch64::*;

// #[cfg(target_arch = "aarch64")]
// mod aarch64;

use pneuma::thread::Thread;

use pneuma::thread::{RcContext};

// #[cfg(all(target_family = "aarch64", target_os = "linux"))]
std::arch::global_asm!(include_str!("asm/aarch64-linux.s"));

extern "C" {
    pub(crate) fn switch_context(store: Thread, next: Thread);
    pub(crate) fn switch_no_save(next: RcContext);
}
