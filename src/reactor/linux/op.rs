use std::{
    io::{self},
    os::fd::AsRawFd,
    time::Duration,
};

use io_uring::{
    opcode,
    types::{Fd, Timespec},
};

use super::{event::submit, Event};

pub fn sleep(dur: Duration) -> io::Result<()> {
    let timespec = Timespec::new().sec(dur.as_secs()).nsec(dur.subsec_nanos());
    let sqe = opcode::Timeout::new(&timespec).build();
    let Err(err) = submit(sqe) else {
        unreachable!();
    };
    if let Some(libc::ETIME) = err.raw_os_error() {
        return Ok(());
    }
    Err(err)
}

#[inline]
pub fn write(fd: &impl AsRawFd, buf: &[u8]) -> io::Result<usize> {
    let sqe = opcode::Write::new(Fd(fd.as_raw_fd()), buf.as_ptr(), buf.len() as _).build();
    let written = submit(sqe)?;
    Ok(written as _)
}

#[inline]
pub fn read(fd: &impl AsRawFd, buf: &mut [u8]) -> io::Result<usize> {
    let sqe = opcode::Read::new(Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _).build();
    let read = submit(sqe)?;
    Ok(read as _)
}

#[inline]
pub fn readable(fd: &impl AsRawFd, flags: u32) -> io::Result<usize> {
    let sqe = opcode::PollAdd::new(Fd(fd.as_raw_fd()), flags)
        .multi(false)
        .build();
    let read = submit(sqe)?;
    Ok(read as _)
}

#[inline]
pub fn readable_multishot(fd: i32) -> Event {
    opcode::PollAdd::new(Fd(fd), libc::POLLIN as _)
        .multi(true)
        .build()
        .user_data(0)
}

#[inline]
pub fn emit_uevent(fd: &impl AsRawFd) -> io::Result<()> {
    write(fd, &1u64.to_ne_bytes())?;
    Ok(())
}
