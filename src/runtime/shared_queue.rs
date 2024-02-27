use std::collections::VecDeque;
use std::io::{self};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pneuma::future::thin_waker::ThinWaker;
use pneuma::net::linux::EventFd;

pub struct SharedQueue {
    pub eventfd: EventFd,
    pub is_sleeping: AtomicBool,
    pub queue: std::sync::Mutex<VecDeque<ThinWaker>>,
}

impl SharedQueue {
    pub fn new() -> io::Result<Arc<Self>> {
        let _thread_id = std::thread::current().id();
        let queue = std::sync::Mutex::default();
        let is_sleeping = AtomicBool::new(false);
        let eventfd = EventFd::new()?;
        let queue = SharedQueue {
            eventfd,
            is_sleeping,
            queue,
        };
        let queue = Arc::new(queue);
        Ok(queue)
    }

    pub fn send(&self, waker: ThinWaker) -> io::Result<()> {
        {
            self.queue.lock().unwrap().push_back(waker);
        }
        if self.is_sleeping() {
            self.eventfd.wake()?;
        }
        Ok(())
    }

    #[inline]
    fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Acquire)
    }

    pub(crate) fn sleep<T>(&self, f: impl FnOnce() -> T) -> T {
        self.is_sleeping.store(true, Ordering::Release);
        let out = f();
        self.is_sleeping.store(false, Ordering::Release);
        out
    }
}
