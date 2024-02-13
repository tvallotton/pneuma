use std::{
    mem::transmute,
    sync::{atomic::Ordering, Arc},
};

use pneuma::thread::{Context, Thread};

use pneuma::runtime::SharedQueue;

pub(crate) struct ThinWaker(Context);

impl ThinWaker {
    pub fn wake_by_ref(&self) {
        self.clone().wake()
    }

    pub fn wake(self) {
        let shared_queue: &'static Arc<SharedQueue> = unsafe { transmute(&self.0.shared_queue) };
        dbg!();
        shared_queue.send(self).unwrap();
    }

    // Must be called from the same thread
    pub unsafe fn into_thread(self) -> Thread {
        self.0.atomic_refcount.fetch_sub(1, Ordering::Release);
        transmute(self)
    }
    /// # Safety
    /// This method must be called from the original thread
    /// where the waker was created
    pub unsafe fn local_wake(self) {
        self.into_thread().unpark()
    }
}

impl Clone for ThinWaker {
    fn clone(&self) -> Self {
        // TODO: understand why thi can be Relaxed
        self.0.atomic_refcount.fetch_add(1, Ordering::Acquire);
        ThinWaker(self.0)
    }
}

impl Drop for ThinWaker {
    fn drop(&mut self) {
        let count = self.0.atomic_refcount.fetch_sub(1, Ordering::Release);
        if count == 1 {
            unsafe { std::alloc::dealloc(self.0 .0.as_ptr().cast(), self.0.layout) }
        }
    }
}

impl From<Thread> for ThinWaker {
    fn from(thread: Thread) -> Self {
        // TODO: understand why the ordering can be Relaxed
        thread.0.atomic_refcount.fetch_add(1, Ordering::Acquire);
        ThinWaker(thread.into_context())
    }
}

unsafe impl Send for ThinWaker {}
unsafe impl Sync for ThinWaker {}
