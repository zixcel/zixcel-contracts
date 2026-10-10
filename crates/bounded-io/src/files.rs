// Derived from Hatter physical intake, using rustix instead of libc/magic flags.
use crate::{
    errors::{exceeded, wrong_type},
    read_bounded,
};
use std::{
    fs::{self, DirEntry, File},
    io,
    path::Path,
};
#[cfg(unix)]
fn open_input(path: &Path) -> io::Result<File> {
    use rustix::fs::{Mode, OFlags, open};
    open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(io::Error::from)
}
#[cfg(windows)]
use crate::windows::open_input;
#[cfg(not(any(unix, windows)))]
fn open_input(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "this platform awaits no-follow validation",
    ))
}
/// Opens a regular input without following the final component; caller owns byte budget.
pub fn open_regular_file(path: impl AsRef<Path>) -> io::Result<File> {
    let path = path.as_ref();
    #[cfg(windows)]
    crate::windows::validate_path(path)?;
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    if crate::windows::rejected_metadata(&metadata) {
        return Err(wrong_type("file"));
    }
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(wrong_type("file"));
    }
    let file = open_input(path)?;
    if !file.metadata()?.is_file() {
        return Err(wrong_type("file"));
    }
    Ok(file)
}
pub fn read_regular_file(path: impl AsRef<Path>, limit: u64) -> io::Result<Vec<u8>> {
    let file = open_regular_file(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > limit {
        return Err(exceeded("bytes", limit, metadata.len()));
    }
    read_bounded(file, limit)
}
pub fn read_regular_text(path: impl AsRef<Path>, limit: u64) -> io::Result<String> {
    String::from_utf8(read_regular_file(path, limit)?)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "input is not UTF-8"))
}
pub fn read_directory(path: impl AsRef<Path>, limit: usize) -> io::Result<Vec<DirEntry>> {
    let path = path.as_ref();
    #[cfg(windows)]
    crate::windows::validate_path(path)?;
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    if crate::windows::rejected_metadata(&metadata) {
        return Err(wrong_type("directory"));
    }
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(wrong_type("directory"));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        if entries.len() == limit {
            return Err(exceeded(
                "entries",
                limit as u64,
                (limit as u64).saturating_add(1),
            ));
        }
        entries.push(entry?);
    }
    Ok(entries)
}
