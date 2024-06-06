#![allow(unreachable_code)]

use super::cstr;
use libc::{mode_t, AT_SYMLINK_NOFOLLOW, S_IFDIR, S_IFLNK, S_IFMT, S_IFREG};
use pneuma::reactor::op;
use pneuma::sys::statx::{statx, statx_timestamp};
use std::ffi::CString;

use std::fs::FileTimes;
use std::io::{self, Error, Result};
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] function
/// or method and represents known metadata about a file such
/// as its permissions, size, modification
/// times, etc.
pub struct Metadata {
    pub(crate) statx: statx,
}

/// Representation of the various permissions on a file.
///
/// This module only currently provides one bit of information,
/// [`Permissions::readonly`], which is exposed on all currently supported
/// platforms. Unix-specific functionality, such as mode bits, is available
/// through the [`PermissionsExt`] trait.
///
/// [`PermissionsExt`]: crate::os::unix::fs::PermissionsExt
#[derive(Clone, PartialEq, Eq)]
pub struct Permissions {
    mode: mode_t,
}

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
#[derive(Clone, Copy)]
pub struct FileType(pub(crate) u16);

/// Given a path, query the file system to get information about a file,
/// directory, etc.
///
/// This function will traverse symbolic links to query information about the
/// destination file.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `statx` function on Unix
/// and the `GetFileInformationByHandle` function on Windows.
/// Note that, this may change in the future.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust,no_run
/// use pneuma::fs;
///
///
/// fn main() -> std::io::Result<()> {
///     let attr = fs::metadata("/some/file/path.txt")?;
///     // inspect attr ...
///     Ok(())
/// }
/// ```
pub fn metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    _metadata(cstr(path)?, 0)
}

/// Query the metadata about a file without following symlinks.
///
/// # Platform-specific behavior
///
/// This function currently corresponds to the `lstat` function on Unix
/// and the `GetFileInformationByHandle` function on Windows.
/// Note that, this may change in the future.
///
/// # Errors
///
/// This function will return an error in the following situations, but is not
/// limited to just these cases:
///
/// * The user lacks permissions to perform `metadata` call on `path`.
/// * `path` does not exist.
///
/// # Examples
///
/// ```rust,no_run
/// use pneuma::fs;
///
///
/// fn main() -> std::io::Result<()> {
///     let attr = fs::symlink_metadata("/some/file/path.txt")?;
///     // inspect attr ...
///     Ok(())
/// }
/// ```
pub fn symlink_metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    _metadata(cstr(path)?, AT_SYMLINK_NOFOLLOW)
}

fn _metadata(path: CString, flags: i32) -> std::io::Result<Metadata> {
    let statx = op::statx(libc::AT_FDCWD, Some(path), flags)?;
    Ok(Metadata { statx })
}

impl Metadata {
    /// Returns the last access time of this metadata.
    ///
    /// The returned value corresponds to the `atime` field of `stat` on Unix
    /// platforms and the `ftLastAccessTime` field on Windows platforms.
    ///
    /// Note that not all platforms will keep this field update in a file's
    /// metadata, for example Windows has an option to disable updating this
    /// time when files are accessed and Linux similarly has `noatime`.
    ///
    /// # Errors
    ///
    /// This field might not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// if let Ok(time) = metadata.accessed() {
    ///     println!("{time:?}");
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(()) }
    /// ```
    pub fn accessed(&self) -> std::io::Result<SystemTime> {
        #[cfg(target_family = "unix")]
        return Ok(system_time(self.statx.stx_atime));
        Err(Error::from(io::ErrorKind::Unsupported))
    }

    /// Returns the creation time listed in this metadata.
    ///
    /// The returned value corresponds to the `btime` field of `statx` on
    /// Linux kernel starting from to 4.11, the `birthtime` field of `stat` on other
    /// Unix platforms, and the `ftCreationTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field might not be available on all platforms, and will return an
    /// `Err` on platforms or filesystems where it is not available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// if let Ok(time) = metadata.created() {
    ///     println!("{time:?}");
    /// } else {
    ///     println!("Not supported on this platform or filesystem");
    /// }
    /// # Ok(()) }
    /// ```
    pub fn created(&self) -> std::io::Result<SystemTime> {
        #[cfg(target_family = "unix")]
        return Ok(system_time(self.statx.stx_ctime));
        Err(Error::from(io::ErrorKind::Unsupported))
    }

    /// Returns the last modification time listed in this metadata.
    ///
    /// The returned value corresponds to the `mtime` field of `stat` on Unix
    /// platforms and the `ftLastWriteTime` field on Windows platforms.
    ///
    /// # Errors
    ///
    /// This field might not be available on all platforms, and will return an
    /// `Err` on platforms where it is not available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("Cargo.toml")?;
    ///
    /// if let Ok(time) = metadata.modified() {
    ///     println!("{time:?}");
    /// } else {
    ///     println!("Not supported on this platform");
    /// }
    /// # Ok(()) }
    /// ```
    pub fn modified(&self) -> io::Result<SystemTime> {
        #[cfg(target_family = "unix")]
        return Ok(system_time(self.statx.stx_mtime));
        Err(Error::from(io::ErrorKind::Unsupported))
    }

    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_file`], and will be false for symlink metadata.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("./target")?;
    /// assert!(!metadata.is_dir());
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.file_type().is_dir()
    }

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_dir`], and will be false for symlink metadata.
    ///
    /// When the goal is simply to read from (or write to) the source, the most
    /// reliable way to test the source can be read (or written to) is to open
    /// it. Only using `is_file` can break workflows like `diff <( prog_a )` on
    /// a Unix-like system for example. See [`File::open`](pneumafs::File::open) or
    /// [`OpenOptions::open`](pneumafs::OpenOptions::open) for more information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("Cargo.lock")?;
    /// assert!(metadata.is_file());
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.file_type().is_file()
    }

    /// Returns the file type for this metadata.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::fs;
    ///
    ///
    /// fn main() -> std::io::Result<()> {
    ///
    ///     let metadata = fs::metadata("foo.txt")?;
    ///
    ///     println!("{:?}", metadata.file_type().is_file());
    ///     Ok(())
    /// }
    /// ```
    pub fn file_type(&self) -> FileType {
        FileType(self.statx.stx_mode)
    }

    /// Returns `true` if this metadata is for a symbolic link.
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.file_type().is_symlink()
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("Cargo.toml")?;
    ///
    /// assert_ne!(0, metadata.len());
    ///
    ///
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.statx.stx_size as usize
    }

    /// Returns the permissions of the file this metadata is for.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::fs;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let metadata = fs::metadata("foo.txt")?;
    ///
    ///     assert!(!metadata.permissions().readonly());
    ///     Ok(())
    /// }
    /// ```
    pub fn permissions(&self) -> Permissions {
        Permissions {
            mode: self.statx.stx_mode as libc::mode_t,
        }
    }
}

impl MetadataExt for Metadata {
    #[allow(deprecated)]
    fn as_raw_stat(&self) -> &std::os::linux::raw::stat {
        unimplemented!();
    }

    fn st_dev(&self) -> u64 {
        let mj = self.statx.stx_dev_major as u64;
        let mn = self.statx.stx_dev_minor as u64;

        ((mj & 0xfffff000) << 32)
            | ((mj & 0x00000fff) << 8)
            | ((mn & 0xffffff00) << 12)
            | (mn & 0x000000ff)
    }
    fn st_ino(&self) -> u64 {
        self.statx.stx_ino as u64
    }
    fn st_mode(&self) -> u32 {
        self.statx.stx_mode as u32
    }
    fn st_nlink(&self) -> u64 {
        self.statx.stx_nlink as u64
    }
    fn st_uid(&self) -> u32 {
        self.statx.stx_uid as u32
    }
    fn st_gid(&self) -> u32 {
        self.statx.stx_gid as u32
    }
    fn st_rdev(&self) -> u64 {
        let mj = self.statx.stx_rdev_major as u64;
        let mn = self.statx.stx_rdev_minor as u64;

        ((mj & 0xfffff000) << 32)
            | ((mj & 0x00000fff) << 8)
            | ((mn & 0xffffff00) << 12)
            | (mn & 0x000000ff)
    }
    fn st_size(&self) -> u64 {
        self.statx.stx_size as u64
    }
    fn st_atime(&self) -> i64 {
        self.statx.stx_atime.tv_sec
    }
    fn st_atime_nsec(&self) -> i64 {
        self.statx.stx_atime.tv_nsec as _
    }
    fn st_mtime(&self) -> i64 {
        self.statx.stx_mtime.tv_sec
    }
    fn st_mtime_nsec(&self) -> i64 {
        self.statx.stx_mtime.tv_nsec as i64
    }
    fn st_ctime(&self) -> i64 {
        self.statx.stx_ctime.tv_sec as i64
    }
    fn st_ctime_nsec(&self) -> i64 {
        self.statx.stx_ctime.tv_nsec as i64
    }
    fn st_blksize(&self) -> u64 {
        self.statx.stx_blksize as u64
    }
    fn st_blocks(&self) -> u64 {
        self.statx.stx_blocks as u64
    }
}

impl std::os::unix::fs::MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        self.st_dev()
    }
    fn ino(&self) -> u64 {
        self.st_ino()
    }
    fn mode(&self) -> u32 {
        self.st_mode()
    }
    fn nlink(&self) -> u64 {
        self.st_nlink()
    }
    fn uid(&self) -> u32 {
        self.st_uid()
    }
    fn gid(&self) -> u32 {
        self.st_gid()
    }
    fn rdev(&self) -> u64 {
        self.st_rdev()
    }
    fn size(&self) -> u64 {
        self.st_size()
    }
    fn atime(&self) -> i64 {
        self.st_atime()
    }
    fn atime_nsec(&self) -> i64 {
        self.st_atime_nsec()
    }
    fn mtime(&self) -> i64 {
        self.st_mtime()
    }
    fn mtime_nsec(&self) -> i64 {
        self.st_mtime_nsec()
    }
    fn ctime(&self) -> i64 {
        self.st_ctime()
    }
    fn ctime_nsec(&self) -> i64 {
        self.st_ctime_nsec()
    }
    fn blksize(&self) -> u64 {
        self.st_blksize()
    }
    fn blocks(&self) -> u64 {
        self.st_blocks()
    }
}

impl FileType {
    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_file`], and will be false for symlink metadata.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("./target")?;
    /// assert!(!metadata.file_type().is_dir());
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn is_dir(&self) -> bool {
        (self.0 as mode_t & S_IFMT) == S_IFDIR
    }

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_dir`], and will be false for symlink metadata.
    ///
    /// When the goal is simply to read from (or write to) the source, the most
    /// reliable way to test the source can be read (or written to) is to open
    /// it. Only using `is_file` can break workflows like `diff <( prog_a )` on
    /// a Unix-like system for example. See [`File::open`](pneumafs::File::open) or
    /// [`OpenOptions::open`](pneumafs::OpenOptions::open) for more information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn __() -> std::io::Result<()> {
    /// use pneuma::fs;
    ///
    /// let metadata = fs::metadata("Cargo.lock")?;
    /// assert!(metadata.file_type().is_file());
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn is_file(&self) -> bool {
        (self.0 as mode_t & S_IFMT) == S_IFREG
    }

    /// Tests whether this file type represents a symbolic link.
    /// The result is mutually exclusive to the results of
    /// [`is_dir`] and [`is_file`]; only zero or one of these
    /// tests may pass.
    ///
    /// The underlying [`Metadata`] struct needs to be retrieved
    /// with the [`fs::symlink_metadata`] function and not the
    /// [`fs::metadata`] function. The [`fs::metadata`] function
    /// follows symbolic links, so [`is_symlink`] would always
    /// return `false` for the target file.
    ///
    /// [`fs::metadata`]: metadata
    /// [`fs::symlink_metadata`]: symlink_metadata
    /// [`is_dir`]: FileType::is_dir
    /// [`is_file`]: FileType::is_file
    /// [`is_symlink`]: FileType::is_symlink
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::fs;
    ///
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let metadata = fs::metadata("foo.txt")?;
    ///     let file_type = metadata.file_type();
    ///
    ///     assert_eq!(file_type.is_symlink(), false);
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        (self.0 as mode_t & S_IFMT) == S_IFLNK
    }

    /// Returns `true` if this file type is a fifo.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::fs;
    /// use std::io;
    ///
    ///
    /// fn main() -> io::Result<()> {
    ///     let meta = fs::metadata("fifo_file")?;
    ///     let file_type = meta.file_type();
    ///     assert!(file_type.is_fifo());
    ///     Ok(())
    /// }
    /// ```
    pub fn is_fifo(&self) -> bool {
        (self.0 as mode_t & libc::S_IFIFO) == libc::S_IFIFO
    }
}

impl Permissions {
    /// Returns `true` if these permissions describe a readonly (unwritable) file.
    ///
    /// # Note
    ///
    /// This function does not take Access Control Lists (ACLs) or Unix group
    /// membership into account.
    ///
    /// # Windows
    ///
    /// On Windows this returns [`FILE_ATTRIBUTE_READONLY`](https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants).
    /// If `FILE_ATTRIBUTE_READONLY` is set then writes to the file will fail
    /// but the user may still have permission to change this flag. If
    /// `FILE_ATTRIBUTE_READONLY` is *not* set then writes may still fail due
    /// to lack of write permission.
    /// The behavior of this attribute for directories depends on the Windows
    /// version.
    ///
    /// # Unix (including macOS)
    ///
    /// On Unix-based platforms this checks if *any* of the owner, group or others
    /// write permission bits are set. It does not check if the current
    /// user is in the file's assigned group. It also does not check ACLs.
    /// Therefore the return value of this function cannot be relied upon
    /// to predict whether attempts to read or write the file will actually succeed.
    /// The [`PermissionsExt`] trait gives direct access to the permission bits but
    /// also does not read ACLs.
    ///
    /// [`PermissionsExt`]: crate::os::unix::fs::PermissionsExt
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let mut f = File::create("foo.txt")?;
    ///     let metadata = f.metadata()?;
    ///
    ///     assert_eq!(false, metadata.permissions().readonly());
    ///     Ok(())
    /// }
    /// ```
    pub fn readonly(&self) -> bool {
        // check if any class (owner, group, others) has write permission
        self.mode & 0o222 == 0
    }

    /// Modifies the readonly flag for this set of permissions. If the
    /// `readonly` argument is `true`, using the resulting `Permission` will
    /// update file permissions to forbid writing. Conversely, if it's `false`,
    /// using the resulting `Permission` will update file permissions to allow
    /// writing.
    ///
    /// This operation does **not** modify the files attributes. This only
    /// changes the in-memory value of these attributes for this `Permissions`
    /// instance. To modify the files attributes use the [`set_permissions`]
    /// function which commits these attribute changes to the file.
    ///
    /// # Note
    ///
    /// `set_readonly(false)` makes the file *world-writable* on Unix.
    /// You can use the [`PermissionsExt`] trait on Unix to avoid this issue.
    ///
    /// It also does not take Access Control Lists (ACLs) or Unix group
    /// membership into account.
    ///
    /// # Windows
    ///
    /// On Windows this sets or clears [`FILE_ATTRIBUTE_READONLY`](https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants).
    /// If `FILE_ATTRIBUTE_READONLY` is set then writes to the file will fail
    /// but the user may still have permission to change this flag. If
    /// `FILE_ATTRIBUTE_READONLY` is *not* set then the write may still fail if
    /// the user does not have permission to write to the file.
    ///
    /// In Windows 7 and earlier this attribute prevents deleting empty
    /// directories. It does not prevent modifying the directory contents.
    /// On later versions of Windows this attribute is ignored for directories.
    ///
    /// # Unix (including macOS)
    ///
    /// On Unix-based platforms this sets or clears the write access bit for
    /// the owner, group *and* others, equivalent to `chmod a+w <file>`
    /// or `chmod a-w <file>` respectively. The latter will grant write access
    /// to all users! You can use the [`PermissionsExt`] trait on Unix
    /// to avoid this issue.
    ///
    /// [`PermissionsExt`]: crate::os::unix::fs::PermissionsExt
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    ///
    /// fn main() -> std::io::Result<()> {
    ///     let f = File::create("foo.txt")?;
    ///     let metadata = f.metadata()?;
    ///     let mut permissions = metadata.permissions();
    ///
    ///     permissions.set_readonly(true);
    ///
    ///     // filesystem doesn't change, only the in memory state of the
    ///     // readonly permission
    ///     assert_eq!(false, metadata.permissions().readonly());
    ///
    ///     // just this particular `permissions`.
    ///     assert_eq!(true, permissions.readonly());
    ///     Ok(())
    /// }
    /// ```
    pub fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            // remove write permission for all classes; equivalent to `chmod a-w <file>`
            self.mode &= !0o222;
        } else {
            // add write permission for all classes; equivalent to `chmod a+w <file>`
            self.mode |= 0o222;
        }
    }

    pub(crate) fn mode(&self) -> libc::mode_t {
        self.mode
    }
}

fn system_time(time: statx_timestamp) -> SystemTime {
    let secs = Duration::from_secs(time.tv_sec as _);
    let nanos = Duration::from_nanos(time.tv_nsec as _);
    SystemTime::UNIX_EPOCH + secs + nanos
}
