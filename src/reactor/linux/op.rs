use std::{
    io::{self, ErrorKind},
    os::fd::AsRawFd,
    time::Duration,
};

use io_uring::{
    opcode,
    types::{Fd, Timespec},
};

use super::event::submit;

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

pub fn write(fd: &impl AsRawFd, buf: &[u8]) -> io::Result<usize> {
    let sqe = opcode::Write::new(Fd(fd.as_raw_fd()), buf.as_ptr(), buf.len() as _).build();
    let written = submit(sqe)?;
    Ok(written as _)
}

pub fn read(fd: &impl AsRawFd, buf: &mut [u8]) -> io::Result<usize> {
    let sqe = opcode::Read::new(Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _).build();
    let read = submit(sqe)?;
    Ok(read as _)
}
