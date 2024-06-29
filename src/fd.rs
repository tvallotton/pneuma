use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};

use pneuma::reactor::op;

pub(crate) struct OwnedFd {
    fd: i32,
}

impl AsRawFd for OwnedFd {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd
    }
}

impl FromRawFd for OwnedFd {
    unsafe fn from_raw_fd(fd: std::os::unix::prelude::RawFd) -> Self {
        OwnedFd { fd }
    }
}

impl IntoRawFd for OwnedFd {
    fn into_raw_fd(self) -> std::os::unix::prelude::RawFd {
        let fd = self.fd;
        std::mem::forget(self);
        fd
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        dbg!(op::close(self.fd));
    }
}
