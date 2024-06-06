use io_uring::squeue;
use io_uring::{
    opcode::{self, OpenAt, Statx},
    types::{Fd, FsyncFlags, Timespec},
};
use pneuma::fd::OwnedFd;
use pneuma::sys::statx::{statx, STATX_BASIC_STATS};
use pneuma::uthread;
use pneuma::utils::IgnorePoison;
use squeue::Entry as Event;

use std::{
    ffi::{CStr, CString},
    io::{self},
    os::fd::{AsRawFd, FromRawFd},
    time::Duration,
};
use std::{io::Error, mem::transmute, sync::atomic::Ordering};

#[track_caller]
pub fn submit(sqe: squeue::Entry) -> io::Result<i32> {
    let thread = pneuma::uthread::current();
    let rt = pneuma::runtime::current();
    let sqe = sqe.user_data(unsafe { transmute(thread.clone()) });

    let mut io_uring = rt.reactor.uring.lock().ignore_poison();

    thread.cx.io_uring_result.store(i64::MAX, Ordering::Relaxed);

    unsafe {
        if io_uring.submission().push(&sqe).is_err() {
            io_uring.submit()?;

            io_uring.submission().push(&sqe).ok();
        }
    }

    drop(io_uring);

    loop {
        let result = thread.cx.io_uring_result.load(Ordering::Relaxed);

        if result == i64::MAX {
            uthread::park();

            continue;
        }

        if result.is_negative() {
            return Err(Error::from_raw_os_error(-result as _));
        }

        return Ok(result as _);
    }
}

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
    write_at(fd.as_raw_fd(), buf, u64::MAX)
}

#[inline]
pub fn read_at(fd: i32, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    let sqe = opcode::Read::new(Fd(fd.as_raw_fd()), buf.as_mut_ptr().cast(), buf.len() as _)
        .offset(offset)
        .build();
    let read = submit(sqe)?;
    Ok(read as _)
}

#[inline]
#[track_caller]
pub fn write_at(fd: i32, buf: &[u8], offset: u64) -> io::Result<usize> {
    let sqe = opcode::Write::new(Fd(fd.as_raw_fd()), buf.as_ptr(), buf.len() as _)
        .offset(offset)
        .build();

    let written = dbg!(submit(sqe))?;

    Ok(written as _)
}

#[inline]
pub fn read(fd: &impl AsRawFd, buf: &mut [u8]) -> io::Result<usize> {
    read_at(fd.as_raw_fd(), buf, u64::MAX)
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
pub fn emit_uevent(fd: i32) -> io::Result<()> {
    write(&fd, &1u64.to_ne_bytes())?;
    Ok(())
}
#[track_caller]
pub fn open_at(path: &CStr, flags: i32, mode: u32) -> io::Result<OwnedFd> {
    let sqe = OpenAt::new(Fd(libc::AT_FDCWD), path.as_ptr())
        .flags(flags)
        .mode(mode)
        .build();
    // Safety: the resource (pathname) is submitted
    let read = dbg!(submit(sqe))?;
    
    Ok(unsafe { OwnedFd::from_raw_fd(read) })
}

pub fn statx(fd: i32, path: Option<CString>, flags: i32) -> io::Result<statx> {
    let pathname = path
        .as_ref()
        .map(|x| x.as_ptr())
        .unwrap_or(b"\0".as_ptr() as *const _);
    let statx = std::mem::MaybeUninit::<statx>::uninit();
    let mut statx = Box::new(statx);
    let sqe = Statx::new(Fd(fd), pathname, statx.as_mut_ptr().cast())
        .mask(STATX_BASIC_STATS)
        .flags(if path.is_none() {
            flags | libc::AT_EMPTY_PATH
        } else {
            flags
        })
        .build();

    submit(sqe)?;

    Ok(unsafe { statx.assume_init_read() })
}

#[track_caller]
pub fn close(fd: i32) -> std::io::Result<i32> {
    let sqe = opcode::Close::new(Fd(fd.as_raw_fd())).build();
    dbg!(std::panic::Location::caller());
    dbg!(submit(sqe))
}

#[track_caller]
pub fn unlink_at(path: &CStr, flags: i32) -> std::io::Result<i32> {
    let sqe = opcode::UnlinkAt::new(Fd(libc::AT_FDCWD), path.as_ptr())
        .flags(flags)
        .build();

    submit(sqe)
}

pub fn fsync(fd: i32) -> io::Result<i32> {
    let sqe = opcode::Fsync::new(Fd(fd)).build();
    submit(sqe)
}
pub fn fsync_data(fd: i32) -> io::Result<i32> {
    let sqe = opcode::Fsync::new(Fd(fd))
        .flags(FsyncFlags::DATASYNC)
        .build();
    submit(sqe)
}

pub fn mkdir_at(path: &CStr) -> io::Result<i32> {
    let sqe = opcode::MkDirAt::new(Fd(libc::AT_FDCWD), path.as_ptr()).build();
    submit(sqe)
}

pub fn symlink_at(target: &CStr, linkpath: &CStr) -> io::Result<()> {
    let sqe =
        opcode::SymlinkAt::new(Fd(libc::AT_FDCWD), target.as_ptr(), linkpath.as_ptr()).build();
    submit(sqe)?;
    Ok(())
}

pub fn link_at(target: &CStr, linkpath: &CStr) -> io::Result<()> {
    let sqe = opcode::LinkAt::new(
        Fd(libc::AT_FDCWD),
        target.as_ptr(),
        Fd(libc::AT_FDCWD),
        linkpath.as_ptr(),
    )
    .build();
    submit(sqe)?;
    Ok(())
}

pub fn rename_at(oldpath: &CStr, newpath: &CStr) -> io::Result<()> {
    let sqe = io_uring::opcode::RenameAt::new(
        Fd(libc::AT_FDCWD),
        oldpath.as_ptr(),
        Fd(libc::AT_FDCWD),
        newpath.as_ptr(),
    )
    .build();
    submit(sqe)?;
    Ok(())
}
