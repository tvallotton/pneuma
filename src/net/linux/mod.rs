use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use pneuma::{reactor, syscall};

use pneuma::reactor::{op, Reactor};

#[cfg(target_os = "linux")]
pub(crate) struct EventFd(OwnedFd);

#[cfg(not(target_os = "linux"))]
pub(crate) struct EventFd(i32);

impl EventFd {
    #[cfg(target_os = "linux")]
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
        reactor::op::emit_uevent(self.0);
        Ok(())
    }
}

impl AsRawFd for EventFd {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.0.as_raw_fd()
    }
}
