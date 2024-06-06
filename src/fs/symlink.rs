use std::{io, path::Path};

use pneuma::reactor::op;

use super::cstr;

pub fn symlink(original: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
    let original = cstr(original)?;
    let link = cstr(link)?;
    op::symlink_at(&original, &link)
}

pub fn read_link(path: impl AsRef<Path>) -> io::Result<std::path::PathBuf> {
    // TODO: move this call to a blocking thread
    std::fs::read_link(path)
}

pub fn hard_link(original: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
    let original = cstr(original)?;
    let link = cstr(link)?;
    op::link_at(&original, &link)
}
