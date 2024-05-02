use std::{io, os::fd::AsRawFd, sync::Mutex, time::Duration};

use mio::Registry;

pub struct Reactor {
    reactor: Mutex<Inner>,
}

pub struct Inner {
    pub poll: mio::Poll,
    pub events: mio::Events,

    #[cfg(target_os = "linux")]
    pub io_uring: io_uring::IoUring,
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
    pub fn new() -> io::Result<Self> {
        todo!()
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
}
