use std::{
    io::{self, Error},
    time::{Duration, Instant},
};

#[inline]
pub fn nonblocking<F, T>(mut closure: F, timeout: Option<Duration>) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    let start = Instant::now();

    loop {
        match closure() {
            Err(e) if e.raw_os_error() == Some(libc::EWOULDBLOCK) => {
                pneuma::uthread::park()?;
            }
            other => return other,
        }

        if timeout.is_some_and(|time| start.elapsed() > time) {
            return Err(Error::from_raw_os_error(libc::ETIMEDOUT));
        }
    }
}
