use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
};

use crate::syscall;

pub struct Sender {
    fd: OwnedFd,
}

pub struct Receiver {
    fd: OwnedFd,
}

pub fn pipe() -> Result<(Sender, Receiver), std::io::Error> {
    let fds = &mut [0, 0];
    syscall!(pipe, fds.as_mut_ptr())?;

    let fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let rx = Receiver { fd };
    let fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let tx = Sender { fd };

    Ok((tx, rx))
}

impl AsRawFd for Sender {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsRawFd for Receiver {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd.as_raw_fd()
    }
}

impl io::Read for Receiver {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        pneuma::reactor::op::read(self, buf)
    }
}

impl io::Write for Sender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        pneuma::reactor::op::write(self, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
