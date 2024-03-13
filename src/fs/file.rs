#![allow(clippy::missing_errors_doc, unused_imports)]

use pneuma::fs::Metadata;
use pneuma::reactor::op;
use pneuma::runtime::current;

use libc::AT_FDCWD;
use std::borrow::Cow;
use std::ffi::{CStr, OsString};
use std::io::{self, Error, Result};
use std::mem::{forget, MaybeUninit};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::path::{Path, PathBuf};

use super::{cstr, OpenOptions};

/// An object providing access to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with.
///
/// Files are automatically closed when they go out of scope.  Errors detected
/// on closing are ignored by the implementation of `Drop`.
///
/// # Examples
///
/// Creates a new file and write bytes to it (you can also use [`write_at()`](File::write_at)):
///
/// ```no_run
/// use std::io::Write;
/// use pneuma::fs::File;
///
/// # fn __() -> std::io::Result<()> {
/// let mut file = File::create("foo.txt")?;
/// write!(file, "Hello world")?;
/// # Ok(())
/// # }
/// ```
pub struct File {
    pub(crate) fd: i32,
}

impl Drop for File {
    fn drop(&mut self) {
        op::close(self.as_raw_fd());
    }
}

impl File {
    /// Attempts to open a file in read-only mode.
    ///
    /// See the [`OpenOptions::open`] method for more details.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    ///
    /// let f = File::open("foo.txt")?;
    /// # Ok(()) }
    /// ```
    pub fn open(path: impl Into<Vec<u8>>) -> Result<File> {
        OpenOptions::new().read(true).open(path)
    }
    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    /// Depending on the platform, this function may fail if the
    /// full directory path does not exist.
    ///
    /// See the [`OpenOptions::open`] function for more details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    ///
    /// let f = File::create("foo.txt")?;
    /// # Ok(()) }
    /// ```
    pub fn create<P: Into<Vec<u8>>>(path: P) -> Result<File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.into())
    }
    /// Creates a new file in read-write mode; error if the file exists.
    ///
    /// This function will create a file if it does not exist, or return an error if it does. This
    /// way, if the call succeeds, the file returned is guaranteed to be new.
    ///
    /// This option is useful because it is atomic. Otherwise between checking whether a file
    /// exists and creating a new one, the file may have been created by another process (a TOCTOU
    /// race condition / attack).
    ///
    /// This can also be written using
    /// `File::options().read(true).write(true).create_new(true).open(...)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    ///
    /// let f = File::create_new("foo.txt")?;
    /// # Ok(()) }
    /// ```
    pub fn create_new(path: impl Into<Vec<u8>>) -> Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
    }

    /// Returns a new `OpenOptions` object.
    ///
    /// This function returns a new `OpenOptions` object that you can use to
    /// open or create a file with specific options if `open()` or `create()`
    /// are not appropriate.
    ///
    /// It is equivalent to `OpenOptions::new()`, but allows you to write more
    /// readable code. Instead of
    /// `OpenOptions::new().append(true).open("example.log")`,
    /// you can write `File::options().append(true).open("example.log")`. This
    /// also avoids the need to import `OpenOptions`.
    ///
    /// See the [`OpenOptions::new`] function for more details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    ///
    /// let f = File::options().append(true).open("example.log")?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Closes the file.
    ///
    /// The method completes once the close operation has completed,
    /// guaranteeing that resources associated with the file have been released.
    ///
    /// If `close` is not called before dropping the file, the file is closed in
    /// the background, but there is no guarantee as to **when** the close
    /// operation will complete. Note that letting a file be closed in the background
    /// incurs in an additional allocation.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    /// use std::io::Write;
    ///
    ///  // open the file
    ///  let f = File::open("foo.txt")?;
    ///  // close the file
    ///  f.close()?;
    /// # Ok(()) }
    /// ```

    pub fn close(self) -> io::Result<()> {
        let fd = self.as_raw_fd();
        forget(self);
        op::close(fd)?;
        Ok(())
    }

    /// Attempts to sync all OS-internal metadata to disk.
    ///
    /// This function will attempt to ensure that all in-memory data reaches the
    /// filesystem before completing.
    ///
    /// This can be used to handle errors that would otherwise only be caught
    /// when the `File` is closed.  Dropping a file will ignore errors in
    /// synchronizing this in-memory data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use std::io::Write;
    /// use pneuma::fs::File;
    ///
    /// let mut f = File::create("foo.txt")?;
    ///
    /// write!(f, "Hello, world!")?;
    ///
    /// f.sync_all()?;
    ///     
    /// // Close the file
    /// f.close()?;
    /// # Ok(()) }
    /// ```
    pub fn sync_all(&mut self) -> Result<()> {
        op::fsync(self.as_raw_fd())?;
        Ok(())
    }

    /// Attempts to sync file data to disk.
    ///
    /// This method is similar to [`sync_all`], except that it may not
    /// synchronize file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don't
    /// need the metadata on disk. The goal of this method is to reduce disk
    /// operations.
    ///
    /// Note that some platforms may simply implement this in terms of
    /// [`sync_all`].
    ///
    /// [`sync_all`]: File::sync_all
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    /// use std::io::Write;
    ///
    /// let mut f = File::create("foo.txt")?;
    ///
    /// write!(f, "Hello, world!")?;
    ///
    /// f.sync_data()?;
    ///
    /// // Close the file
    /// f.close()?;
    /// # Ok(()) }
    /// ```
    pub fn sync_data(&mut self) -> Result<()> {
        op::fsync_data(self.as_raw_fd())?;
        Ok(())
    }

    /// Queries metadata about the underlying file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::Write;
    ///
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs::File;
    ///
    /// let f = File::open("foo.txt")?;
    /// let metadata = f.metadata()?;
    /// assert!(metadata.is_file());
    /// # Ok(()) }
    /// ```
    pub fn metadata(&self) -> Result<Metadata> {
        let statx = op::statx(self.as_raw_fd(), None, 0)?;
        Ok(Metadata { statx })
    }
    /// Destructures `File` into a [`std::fs::File`].
    pub fn into_std(self) -> std::fs::File {
        let fd = self.into_raw_fd();
        unsafe { std::fs::File::from_raw_fd(fd) }
    }

    /// Converts a [`std::fs::File`] to a [`pneuma::fs::File`](File).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // This line could block. It is not recommended to do this on the Tokio
    /// // runtime.
    /// let std_file = std::fs::File::open("foo.txt").unwrap();
    /// let file = pneuma::fs::File::from_std(std_file);
    /// ```

    pub fn from_std(file: std::fs::File) -> Self {
        let fd = file.into_raw_fd();
        File { fd }
    }
}

/// Removes a file from the filesystem.
///
/// Note that there is no
/// guarantee that the file is immediately deleted (e.g., depending on
/// platform, other open file descriptors may prevent immediate removal).
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `unlink` function on Unix
/// and the `DeleteFile` function on Windows.
/// Note that, this [may change in the future][changes].
///
/// [changes]: io#platform-specific-behavior
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * `path` points to a directory.
/// * The file doesn't exist.
/// * The user lacks permissions to remove the file.
///
/// # Examples
///
/// ```no_run
/// # fn __() -> std::io::Result<()> {
/// use pneuma::fs;
///
/// fs::remove_file("foo.txt")?;
/// # Ok(()) }
/// ```
pub fn remove_file(path: impl Into<Vec<u8>>) -> Result<()> {
    let path = cstr(path)?;
    op::unlink_at(&path, 0)?;
    Ok(())
}

impl From<std::fs::File> for File {
    fn from(value: std::fs::File) -> Self {
        unsafe { File::from_raw_fd(value.into_raw_fd()) }
    }
}

impl From<File> for std::fs::File {
    fn from(value: File) -> Self {
        unsafe { Self::from_raw_fd(value.into_raw_fd()) }
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        op::write(self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        op::read(self, buf)
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> std::os::fd::RawFd {
        let fd = self.fd;
        forget(self);
        fd
    }
}

impl AsRawFd for File {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.fd
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
        File { fd }
    }
}
