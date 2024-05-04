use std::{
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_UTHREAD: AtomicU64 = AtomicU64::new(0);

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct UThreadId(NonZeroU64);

impl UThreadId {
    pub(crate) fn new() -> Self {
        let uthread = MAX_UTHREAD.fetch_add(1, Ordering::Relaxed);
        let uthread = unsafe { NonZeroU64::new_unchecked(uthread) };
        UThreadId(uthread)
    }

    pub fn as_u64(self) -> u64 {
        self.0.get()
    }
}
