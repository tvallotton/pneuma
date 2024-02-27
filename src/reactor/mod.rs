use std::{cell::RefCell, io, os::fd::AsRawFd, sync::Arc};

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub(crate) use bsd as imp;
pub use imp::op;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) use linux as imp;

use pneuma::runtime::SharedQueue;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub(crate) mod bsd;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) mod linux;

// #[cfg(not(target_os = "linux"))]
// pub(crate) mod non_linux;

pub struct Reactor(RefCell<imp::Reactor>);

impl Reactor {
    pub fn new(shared_queue: Arc<SharedQueue>) -> io::Result<Reactor> {
        let reactor = RefCell::new(imp::Reactor::new()?);
        let reactor = Reactor(reactor);
        reactor.setup_crossthread_wakups(shared_queue)?;
        Ok(reactor)
    }

    pub fn push(&self, ev: imp::Event) -> io::Result<()> {
        borrow_mut!(self.0).push(ev)
    }

    pub fn submit_and_yield(&self) -> io::Result<()> {
        borrow_mut!(self.0).submit_and_yield()
    }

    pub fn submit_and_wait(&self) -> io::Result<()> {
        borrow_mut!(self.0).submit_and_wait()
    }

    fn setup_crossthread_wakups(&self, shared_queue: Arc<SharedQueue>) -> io::Result<()> {
        let event = op::readable_multishot(shared_queue.eventfd.as_raw_fd());
        self.push(event)
    }
}
