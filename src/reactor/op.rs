use mio::Interest;
use std::io::{self, Write};
use std::io::{IoSlice, Read};

use super::registration::Registration;

#[inline]
pub fn nonblocking<F, T>(mut closure: F) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    loop {
        match closure() {
            Err(e) if e.raw_os_error() == Some(libc::EWOULDBLOCK) => {
                pneuma::uthread::park()?;
            }
            other => return other,
        }
    }
}
