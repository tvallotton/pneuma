use mio::Interest;
pub use registration::Registration;
use slab::Slab;
use std::{
    io,
    sync::{atomic::AtomicU8, Mutex},
    time::Duration,
};

use crate::uthread::UThread;

pub mod op;
mod registration;

pub struct Reactor {
    pub reactor: Mutex<Inner>,
}

pub struct Inner {
    pub poll: mio::Poll,
    pub events: mio::Events,

    pub tokens: Slab<UThread>,
    #[cfg(target_os = "linux")]
    pub io_uring: io_uring::IoUring,
}

impl Reactor {
    #[cfg(target_os = "linux")]
    pub fn new() -> io::Result<Self> {
        let io_uring = io_uring::IoUring::new(256)?;
        let poll = mio::Poll::new()?;
        let events = mio::Events::with_capacity(256);
        let tokens = Slab::new();
        let reactor = Inner {
            poll,
            events,
            tokens,
            io_uring,
        };

        let reactor = Mutex::new(reactor);
        Ok(Reactor { reactor })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> io::Result<Self> {
        let poll = mio::Poll::new()?;
        let events = mio::Events::with_capacity(256);
        let tokens = Slab::new();
        let reactor = Inner {
            poll,
            events,
            tokens,
        };

        let reactor = Mutex::new(reactor);
        Ok(Reactor { reactor })
    }

    pub fn submit_and_yield(&self) -> io::Result<()> {
        self.submit(Some(Duration::ZERO))
    }

    pub fn submit_and_wait(&self) -> io::Result<()> {
        self.submit(None)
    }

    fn submit(&self, timeout: Option<Duration>) -> io::Result<()> {
        let reactor = &mut *self.reactor.lock().unwrap();
        #[cfg(target_os = "linux")]
        reactor.io_uring.submit()?;
        let Inner { poll, events, .. } = reactor;
        poll.poll(events, timeout)?;

        for event in events.iter() {
            let Some(uthread) = reactor.tokens.get(event.token().0) else {
                continue;
            };
            uthread.unpark();
        }

        Ok(())
    }
}

pub fn current() -> &'static Reactor {
    &pneuma::runtime::current().reactor
}
