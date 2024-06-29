use crate::fs::File;
use std::io::{self, Error, ErrorKind, Read, Result};
use std::path::Path;

/// Read the entire contents of a file into a bytes vector.
///
/// This is a convenience function for using [`File::open`], [`File::metadata`] and [`File::read_at`]
/// with fewer imports and without an intermediate variable.
///
/// [`read_to_end`]: Read::read_to_end
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`](crate::fs::OpenOptions::open).
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`io::ErrorKind::Interrupted`].
///
/// # Examples
///
/// ```no_run
/// # let _ = || {
/// use pneuma::fs;
/// use std::net::SocketAddr;
///
/// let foo: SocketAddr = String::from_utf8_lossy(&fs::read("address.txt")?).parse()?;
/// # Ok::<(), Box<dyn std::error::Error>>(()) };
/// ```
pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    _read(path.as_ref())
}

fn _read(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let mut buf = Vec::with_capacity(len as _);
    let _ = file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Read the entire contents of a file into a string.
///
/// This is a convenience function for using [`read`] and [`String::from_utf8`]
/// with fewer imports and without an intermediate variable.
///
/// [`read_to_string`]: Read::read_to_string
///
/// # Errors
///
/// This function will return an error if `path` does not already exist.
/// Other errors may also be returned according to [`OpenOptions::open`](crate::fs::OpenOptions::open).
///
/// It will also return an error if it encounters while reading an error
/// of a kind other than [`io::ErrorKind::Interrupted`],
/// or if the contents of the file are not valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// # let _ = || {
/// use pneuma::fs;
/// use std::net::SocketAddr;
/// use std::error::Error;
///
/// let foo: SocketAddr = fs::read_to_string("address.txt")?.parse()?;
/// # Ok::<(), Box<dyn std::error::Error>>(()) };
/// ```
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    _read_to_string(path.as_ref())
}

fn _read_to_string(path: &Path) -> io::Result<String> {
    let bytes = _read(path)?;
    match String::from_utf8(bytes) {
        Ok(str) => Ok(str),
        Err(_) => Err({
            Error::new(
                ErrorKind::InvalidData,
                "the contents of the file were not valid utf-8.",
            )
        }),
    }
}
