use std::{
    io,
    path::{Path, PathBuf},
};

use pneuma::reactor::op;

use super::cstr;

pub struct ReadDir {
    inner: std::fs::ReadDir,
}

pub struct DirEntry {
    inner: std::fs::DirEntry,
}

pub fn create_dir(path: impl AsRef<Path>) -> io::Result<()> {
    let path = cstr(path)?;
    op::mkdir_at(&path)?;
    Ok(())
}

pub fn remove_dir(path: impl AsRef<Path>) -> io::Result<()> {
    let path = cstr(path)?;
    op::unlink_at(&path, libc::AT_REMOVEDIR)?;
    Ok(())
}

pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if path == Path::new("") {
        return Ok(());
    }

    match create_dir(path) {
        Ok(()) => return Ok(()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(_) if path.is_dir() => return Ok(()),
        Err(e) => return Err(e),
    }

    match path.parent() {
        Some(p) => create_dir_all(p)?,
        None => {
            return Err(io::Error::other("failed to create whole tree"));
        }
    }
    match create_dir(path) {
        Ok(()) => Ok(()),
        Err(_) if path.is_dir() => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    let filetype = pneuma::fs::symlink_metadata(path)?.file_type();
    if filetype.is_symlink() {
        pneuma::fs::remove_file(path)
    } else {
        remove_dir_all_recursive(path)
    }
}

fn remove_dir_all_recursive(path: &Path) -> io::Result<()> {
    for child in std::fs::read_dir(path)? {
        let child = child?;
        if child.file_type()?.is_dir() {
            remove_dir_all_recursive(&child.path())?;
        } else {
            pneuma::fs::remove_file(&child.path())?;
        }
    }
    remove_dir(path)
}

// TODO: forward to a blocking threadpool
pub fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    Ok(ReadDir {
        inner: std::fs::read_dir(path)?,
    })
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.next()?;
        Some(match result {
            Ok(inner) => Ok(DirEntry { inner }),
            Err(err) => Err(err),
        })
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.inner.path()
    }

    /// Returns the file type for the file that this entry points at.
    ///
    /// This function will not traverse symlinks if this entry points at a
    /// symlink.
    ///
    /// # Platform-specific behavior
    ///
    /// On Windows and most Unix platforms this function is free (no extra
    /// system calls needed), but some Unix platforms may require the equivalent
    /// call to `symlink_metadata` to learn about the target file type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    ///
    /// if let Ok(entries) = fs::read_dir(".") {
    ///     for entry in entries {
    ///         if let Ok(entry) = entry {
    ///             // Here, `entry` is a `DirEntry`.
    ///             if let Ok(file_type) = entry.file_type() {
    ///                 // Now let's show our entry's file type!
    ///                 println!("{:?}: {:?}", entry.path(), file_type);
    ///             } else {
    ///                 println!("Couldn't get file type for {:?}", entry.path());
    ///             }
    ///         }
    ///     }
    /// }
    /// ```

    pub fn file_type(&self) -> io::Result<std::fs::FileType> {
        self.inner.file_type()
    }
}
