use io_uring::types::Fixed;
use mio::event::Event;
use mio::Events;
use pneuma::uthread::UThread;
use pneuma::utils::IgnorePoison;
pub use registered::Registered;
use slab::Slab;
#[cfg(target_os = "linux")]
use std::sync::atomic::Ordering::Relaxed;
use std::sync::TryLockError;
use std::time::Instant;
use std::{io, sync::Mutex, time::Duration};
use std::{mem::transmute, sync::Condvar};

pub mod nonblocking;
mod registered;
pub mod uring;

pub mod op {
    pub use super::nonblocking::*;
    pub use super::uring::*;
}

pub struct Reactor {
    pub waker: Waker,

    /// the remaining fields will be used to submit and retrieve events.
    pub mio: Mutex<Mio>,
    pub uring: Mutex<io_uring::IoUring>,
}

pub struct Waker {
    pub is_blocking: Mutex<bool>,
    pub condvar: Condvar,

    pub poll: Mutex<(Events, mio::Poll)>,
}

pub struct Mio {
    pub events: mio::Events,
    pub poll: mio::Poll,
}

pub struct Inner {
    pub events: mio::Events,

    pub epoll: mio::Poll,
    #[cfg(target_os = "linux")]
    pub io_uring: io_uring::IoUring,
}

impl Reactor {
    #[cfg(target_os = "linux")]
    pub fn new() -> io::Result<Self> {
        use mio::{unix::SourceFd, Interest, Token};
        use std::os::fd::AsRawFd;

        let poll = mio::Poll::new()?;
        let uring = io_uring::IoUring::new(32)?;
        let mio = Mio {
            poll: mio::Poll::new()?,
            events: mio::Events::with_capacity(256),
        };

        // Register io-uring on epoll
        poll.registry().register(
            &mut SourceFd(&uring.as_raw_fd()),
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        // Register epoll on epoll
        poll.registry().register(
            &mut SourceFd(&mio.poll.as_raw_fd()),
            Token(1),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let waker = Waker {
            is_blocking: Mutex::new(false),
            condvar: Condvar::new(),
            poll: Mutex::new((Events::with_capacity(2), poll)),
        };

        let reactor = Reactor {
            waker,
            mio: Mutex::new(mio),
            uring: Mutex::new(uring),
        };

        Ok(reactor)
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
        self.submit(Some(Duration::from_millis(100)))
    }

    fn submit(&self, timeout: Option<Duration>) -> io::Result<()> {
        let mut is_blocking = self.waker.is_blocking.lock().ignore_poison();

        if *is_blocking {
            self.waker.condvar.wait(is_blocking).ignore_poison();

            return Ok(());
        }

        *is_blocking = true;
        drop(is_blocking);

        self.uring.lock().ignore_poison().submit()?;
        let mut guard = self.waker.poll.lock().ignore_poison();
        let (events, poll) = &mut *guard;
        let result = poll.poll(events, timeout);

        let mut is_blocking = self.waker.is_blocking.lock().ignore_poison();
        *is_blocking = false;

        for event in events.iter() {
            match event.token().0 {
                0 => self.unpark_uring(),
                _ => self.unpark_mio(),
            }
        }
        drop(guard);
        drop(is_blocking);

        self.waker.condvar.notify_all();

        result
    }

    pub fn unpark_mio(&self) {
        let Mio { events, .. } = &mut *self.mio.lock().ignore_poison();
        for event in events.iter() {
            if event.token().0 == 0 {
                continue;
            }
            let uthread: &UThread = unsafe { transmute(&event.token().0) };

            uthread.unpark();
        }
    }

    #[cfg(target_os = "linux")]
    pub fn unpark_uring(&self) {
        for event in self.uring.lock().ignore_poison().completion() {
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
