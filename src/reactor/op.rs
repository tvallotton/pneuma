use std::io::{self, Write};

use mio::Interest;

use super::registration::Registration;

pub fn write<W>(writer: &mut W, registration: &Registration, buf: &[u8]) -> io::Result<usize>
where
    W: mio::event::Source + Write,
{
    if !registration.interests().is_writable() {
        registration.reregister(writer, registration.interests() | Interest::WRITABLE)?;
    }

    loop {
        match writer.write(buf) {
            Err(e) if e.raw_os_error() == Some(libc::EWOULDBLOCK) => {
                pneuma::uthread::park()?;
            }
            other => return other,
        }
    }
}
