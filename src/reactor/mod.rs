use crate::uthread::UThread;
pub use registered::Registered;
use slab::Slab;
use std::mem::transmute;
#[cfg(target_os = "linux")]
use std::sync::atomic::Ordering::Relaxed;
use std::{io, sync::Mutex, time::Duration};

pub mod nonblocking;
mod registered;
pub mod uring;

pub mod op {
    pub use super::nonblocking::*;
    pub use super::uring::*;
}

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
        use mio::{unix::SourceFd, Interest, Token};
        use std::os::fd::AsRawFd;

        let io_uring = io_uring::IoUring::new(256)?;
        let poll = mio::Poll::new()?;
        let events = mio::Events::with_capacity(256);
        let tokens = Slab::new();

        // Register io-uring on epoll
        poll.registry().register(
            &mut SourceFd(&io_uring.as_raw_fd()),
            Token(usize::MAX),
            Interest::READABLE,
        )?;

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
        self.reactor.lock().unwrap().submit(timeout)
    }
}

impl Inner {
    pub fn submit(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        self.io_uring.submit()?;

        self.poll.poll(&mut self.events, timeout)?;

        #[cfg(target_os = "linux")]
        self.unpark_uring();

        self.unpark_mio();

        Ok(())
    }

    pub fn unpark_mio(&mut self) {
        let Inner { events, .. } = self;
        for event in events.iter() {
            let uthread: &UThread = unsafe { transmute(&event.token().0) };
            uthread.unpark();
        }
    }

    #[cfg(target_os = "linux")]
    pub fn unpark_uring(&mut self) {
        for event in self.io_uring.completion() {
            let uthread: UThread = unsafe { transmute(event.user_data()) };
            uthread
                .cx
                .io_uring_result
                .store(event.result() as i64, Relaxed);
            uthread.unpark();
        }
    }
}

pub fn current() -> &'static Reactor {
    &pneuma::runtime::current().reactor
}
