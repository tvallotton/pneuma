use std::{
    ffi::CString,
    io::{self, Error, Result, Write},
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};

pub use file::{remove_file, File};
pub use metadata::{metadata, symlink_metadata, Metadata};
pub use open_options::OpenOptions;

mod file;
mod metadata;
mod open_options;

pub(crate) fn cstr<P>(path: P) -> Result<CString>
where
    P: AsRef<Path>,
{
    CString::new(path.as_ref().as_os_str().as_encoded_bytes()).map_err(Error::other)
}

/// Write a slice as the entire contents of a file.
///
/// This function will create a file if it does not exist,
/// and will entirely replace its contents if it does.
///
/// Depending on the platform, this function may fail if the
/// full directory path does not exist.
///
/// This is a convenience function for using [`File::create`] and [`write_all`]
/// with fewer imports.
///
/// [`write_all`]: Write::write_all
///
/// # Examples
///
/// ```no_run
/// use pneuma::fs;
///
/// fn main() -> std::io::Result<()> {
///     fs::write("foo.txt", b"Lorem ipsum")?;
///     fs::write("bar.txt", "dolor sit")?;
///     Ok(())
/// }
/// ```

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    fn inner(path: &Path, contents: &[u8]) -> io::Result<()> {
        File::create(path)?.write_all(contents)
    }
    inner(path.as_ref(), contents.as_ref())
}
