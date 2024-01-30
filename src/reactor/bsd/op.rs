use std::{
    cell::Cell,
    io,
    io::Error,
    os::fd::AsRawFd,
    ptr::null_mut,
    time::{Duration, Instant},
};

use libc::kevent;

use crate::{syscall};

use super::event::submit;

const ZEROED: kevent = kevent {
    ident: 0,
    filter: 0,
    flags: 0,
    fflags: 0,
    data: 0,
    udata: null_mut(),
};

#[inline]
pub fn sleep(dur: Duration) -> io::Result<()> {
    let mut event = ZEROED;
    event.ident += event_id();
    event.flags = libc::EV_ADD | libc::EV_ENABLE;
    event.filter = libc::EVFILT_TIMER;
    event.data = dur.as_millis() as _;
    let time = Instant::now();
    submit(event, || {
        if time.elapsed() < dur {
            Err(Error::from_raw_os_error(libc::EAGAIN))
        } else {
            Ok(())
        }
    })
}

#[inline]
pub fn write(fd: &mut impl AsRawFd, buf: &[u8]) -> io::Result<usize> {
    let fd = fd.as_raw_fd();
    let count = buf.len();
    let buf = buf.as_ptr().cast();

    let mut event = ZEROED;
    event.ident = fd as _;
    event.flags = libc::EV_ADD | libc::EV_ONESHOT;

    let written = submit(event, || syscall!(write, fd, buf, count))?;
    Ok(written as usize)
}

#[inline]
pub fn read(fd: &mut impl AsRawFd, buf: &mut [u8]) -> io::Result<usize> {
    let fd = fd.as_raw_fd();
    let count = buf.len();
    let buf = buf.as_mut_ptr().cast();

    let mut event = ZEROED;
    event.ident = fd as _;
    event.flags = libc::EV_ADD | libc::EV_ONESHOT;

    let read = submit(event, || syscall!(write, fd, buf, count))?;
    Ok(read as _)
}

fn event_id() -> usize {
    thread_local! {
        static EVENT_ID: Cell<usize> = Cell::default();
    }
    EVENT_ID.with(|cell| {
        let value = cell.get();
        cell.set(value + 1);
        value
    })
}
