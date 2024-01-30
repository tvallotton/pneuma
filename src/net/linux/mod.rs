use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    time::SystemTime,
};

use pneuma::{reactor, syscall};

use crate::reactor::{imp::Event, op, Reactor};

#[cfg(target_os = "linux")]
pub(crate) struct EventFd(OwnedFd);

#[cfg(not(target_os = "linux"))]
pub(crate) struct EventFd(i32);

impl EventFd {
    pub fn new() -> io::Result<Self> {
        let fd = syscall!(eventfd, 0, 0)?;
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        Ok(EventFd(fd))
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> io::Result<Self> {
        Ok(EventFd(unsafe { libc::rand() }))
    }

    pub fn wake(&self) -> io::Result<()> {
        let buf = 1u64.to_ne_bytes();
        reactor::op::write(self, &buf)?;
        Ok(())
    }

    pub fn register_multishot(&self, reactor: &Reactor) -> io::Result<()> {
        let event = op::readable_multishot(self, 0);
        reactor.push(event)
    }
}

impl AsRawFd for EventFd {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}
