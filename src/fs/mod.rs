use std::{
    ffi::CString,
    io::{self, BufReader, BufWriter, Error, ErrorKind, Read, Result, Write},
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
};

use crate::reactor::op;
pub use dir::{create_dir, create_dir_all, read_dir, remove_dir, remove_dir_all, ReadDir};
pub use file::{remove_file, File};
pub use metadata::{metadata, symlink_metadata, FileType, Metadata};
pub use open_options::OpenOptions;
pub use read::{read, read_to_string};
pub use std::fs::FileTimes;
pub use symlink::{hard_link, read_link, symlink};

mod dir;
mod file;
mod metadata;
mod open_options;
mod read;
mod symlink;

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

/// Returns the canonical, absolute form of a path with all intermediate
/// components normalized and symbolic links resolved.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `realpath` function on Unix
/// and the `CreateFile` and `GetFinalPathNameByHandle` functions on Windows.
/// Note that, this [may change in the future][changes].
///
/// On Windows, this converts the path to use [extended length path][path]
/// syntax, which allows your program to use longer path names, but means you
/// can only join backslash-delimited paths to it, and it may be incompatible
/// with other applications (if passed to the application on the command-line,
/// or written to a file another application may read).
///
/// [changes]: io#platform-specific-behavior
/// [path]: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` does not exist.
/// * A non-final component in path is not a directory.
///
/// # Examples
///
/// ```no_run
/// use pneuma::fs;
///
/// fn main() -> std::io::Result<()> {
///     let path = fs::canonicalize("../a/../foo.txt")?;
///     Ok(())
/// }
/// ```
pub fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    // TODO: send this to a blocking threadpool
    std::fs::canonicalize(path)
}

/// Rename a file or directory to a new name, replacing the original file if
/// `to` already exists.
///
/// This will not work if the new name is on a different mount point.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `rename` function on Unix
/// and the `MoveFileEx` function with the `MOVEFILE_REPLACE_EXISTING` flag on Windows.
///
/// Because of this, the behavior when both `from` and `to` exist differs. On
/// Unix, if `from` is a directory, `to` must also be an (empty) directory. If
/// `from` is not a directory, `to` must also be not a directory. In contrast,
/// on Windows, `from` can be anything, but `to` must *not* be a directory.
///
/// Note that, this [may change in the future][changes].
///
/// [changes]: io#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `from` does not exist.
/// * The user lacks permissions to view contents.
/// * `from` and `to` are on separate filesystems.
///
/// # Examples
///
/// ```no_run
/// use pneuma::fs;
///
/// fn main() -> std::io::Result<()> {
///     fs::rename("a.txt", "b.txt")?; // Rename a.txt to b.txt
///     Ok(())
/// }
/// ```
#[doc(alias = "mv", alias = "MoveFile", alias = "MoveFileEx")]
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
    op::rename_at(&cstr(from)?, &cstr(to)?)
}
