use std::{
    io,
    mem::transmute,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    time::Duration,
};

use crate::{syscall, thread::Thread};

mod event;
pub mod op;

pub struct Reactor {
    fd: OwnedFd,
    list: Vec<libc::kevent>,
}

pub use libc::kevent as Event;

impl Reactor {
    pub fn new() -> io::Result<Reactor> {
        let fd = syscall!(kqueue)?;
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let list = Vec::with_capacity(1024);
        Ok(Reactor { fd, list })
    }

    #[inline]
    pub fn push(&mut self, ev: libc::kevent) -> io::Result<()> {
        self.list.push(ev);
        if self.list.len() >= 512 {
            return self.submit_and_yield();
        }
        Ok(())
    }

    #[inline]
    pub fn submit_and_wait(&mut self) -> io::Result<()> {
        self.submit(Duration::from_secs(8))
    }

    #[inline]
    pub fn submit_and_yield(&mut self) -> io::Result<()> {
        self.submit(Duration::ZERO)
    }

    #[inline]
    fn submit(&mut self, wait: Duration) -> io::Result<()> {
        let changelist = self.list.as_mut_ptr().cast();
        let timespec = libc::timespec {
            tv_sec: wait.as_secs() as _,
            tv_nsec: wait.subsec_nanos() as _,
        };
        let len = syscall!(
            kevent,
            self.fd.as_raw_fd(),
            changelist,
            self.list.len() as _,
            changelist,
            self.list.len() as _,
            &timespec
        )?;
        unsafe { self.list.set_len(len as _) };
        self.wake_all();
        Ok(())
    }

    #[inline]
    pub fn wake_all(&mut self) {
        for event in &mut self.list {
            let thread: Thread = unsafe { transmute(event.udata) };
            thread.unpark();
        }
    }
}
