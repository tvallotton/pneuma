use pneuma::net::unix::pipe::{pipe, Receiver, Sender};
use pneuma::sync::Mutex;
use pneuma::thread::Thread;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::mem::transmute;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::ThreadId;

use crate::future::thin_waker::ThinWaker;
use crate::net::linux::EventFd;
use crate::reactor::Reactor;

pub struct SharedQueue {
    eventfd: EventFd,
    is_sleeping: AtomicBool,
    pub queue: std::sync::Mutex<VecDeque<ThinWaker>>,
}

impl SharedQueue {
    pub fn new(reactor: &Reactor) -> io::Result<Arc<Self>> {
        let thread_id = std::thread::current().id();
        let queue = std::sync::Mutex::default();
        let is_sleeping = AtomicBool::new(false);
        let eventfd = EventFd::new()?;

        eventfd.register_multishot(&reactor)?;
        let queue = SharedQueue {
            eventfd,
            is_sleeping,
            queue,
        };
        let queue = Arc::new(queue);
        Ok(queue)
    }

    pub fn send(&self, waker: ThinWaker) -> io::Result<()> {
        self.queue.lock().unwrap().push_back(waker);

        if self.is_sleeping() {
            self.eventfd.wake()?;
        }

        Ok(())
    }

    #[inline]
    fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Acquire)
    }
}
