use std::{io, sync::Mutex, time::Duration};

use slab::Slab;

use crate::uthread::UThread;


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


pub struct UnregisterGuard {
    key: usize,
}


impl Reactor {
    #[cfg(target_os = "linux")]
    pub fn new() -> io::Result<Self> {
        let io_uring = io_uring::IoUring::new(256)?;
        let poll = mio::Poll::new()?;
        let events = mio::Events::with_capacity(256);

        let reactor = Inner {
            poll,
            events,
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
        let reactor = Inner { poll, events, tokens };

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
        poll.poll(events, timeout)
    }

    #[rustfmt::skip]
    pub fn register<S>(&self, source: &mut S, interests: mio::Interest) -> io::Result<UnregisterGuard>
    where
        S: mio::event::Source + ?Sized,
    {
        let uthread = pneuma::uthread::current();
        let mut reactor = self.reactor.lock().unwrap();

        let key = reactor.tokens.insert(uthread);

        let guard = UnregisterGuard { key };
        
        reactor
            .poll
            .registry()
            .register(source, mio::Token(key), interests)
            .map(|_| guard)
    }
}

impl Drop for UnregisterGuard {
    fn drop(&mut self) {
        pneuma::runtime::current()
            .reactor
            .reactor
            .lock()
            .unwrap()
            .tokens
            .remove(self.key);
    }
}

pub fn current() -> &'static Reactor {
    &pneuma::runtime::current().reactor
}