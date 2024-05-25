use mio::Interest;
use std::io::Read;
use std::io::{self, Write};

use super::registration::Registration;

pub fn write<W>(writer: &mut W, registration: &mut Registration, buf: &[u8]) -> io::Result<usize>
where
    W: mio::event::Source + Write,
{
    if !registration.interests().is_readable() {
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

pub fn read<R>(reader: &mut R, registration: &mut Registration, buf: &mut [u8]) -> io::Result<usize>
where
    R: mio::event::Source + Read,
{
    if !registration.interests().is_writable() {
        registration.reregister(reader, registration.interests() | Interest::WRITABLE)?;
    }

    loop {
        match reader.read(buf) {
            Err(e) if e.raw_os_error() == Some(libc::EWOULDBLOCK) => {
                pneuma::uthread::park()?;
            }
            other => return other,
        }
    }
}
