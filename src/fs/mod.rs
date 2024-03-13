use std::{
    ffi::CString,
    io::{Error, Result},
};

pub use file::{remove_file, File};
pub use metadata::{metadata, symlink_metadata, Metadata};
pub use open_options::OpenOptions;

mod file;
mod metadata;
mod open_options;

pub(crate) fn cstr(path: impl Into<Vec<u8>>) -> Result<CString> {
    CString::new(path.into()).map_err(Error::other)
}
