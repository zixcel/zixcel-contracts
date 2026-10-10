//! Standard Windows open options plus mature winapi-util safe handle inspection.
use crate::errors::wrong_type;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::{
    fs::{File, Metadata, OpenOptions},
    io,
    path::{Component, Path, Prefix},
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DEVICE, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
};
pub(crate) fn validate_path(path: &Path) -> io::Result<()> {
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::DeviceNS(_) | Prefix::Verbatim(_) => return Err(wrong_type("file")),
                Prefix::UNC(_, share) | Prefix::VerbatimUNC(_, share)
                    if share.to_string_lossy().eq_ignore_ascii_case("pipe") =>
                {
                    return Err(wrong_type("file"));
                }
                _ => {}
            },
            Component::Normal(name) => {
                let upper = name.to_string_lossy().to_ascii_uppercase();
                let base = upper
                    .split(['.', ':'])
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(' ');
                let numbered_device = base
                    .strip_prefix("COM")
                    .or_else(|| base.strip_prefix("LPT"))
                    .is_some_and(|n| {
                        matches!(
                            n,
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                        )
                    });
                if numbered_device
                    || matches!(base, "NUL" | "CON" | "PRN" | "AUX" | "CONIN$" | "CONOUT$")
                {
                    return Err(wrong_type("file"));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
pub(crate) fn rejected_metadata(metadata: &Metadata) -> bool {
    metadata.file_attributes() & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DEVICE) != 0
}
pub(crate) fn open_input(path: &Path) -> io::Result<File> {
    validate_path(path)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        // SECURITY_ANONYMOUS=0; standard API also adds SECURITY_SQOS_PRESENT.
        .security_qos_flags(0)
        .open(path)?;
    if !winapi_util::file::typ(&file)?.is_disk() {
        return Err(wrong_type("file"));
    }
    if rejected_metadata(&file.metadata()?) {
        return Err(wrong_type("file"));
    }
    Ok(file)
}
#[cfg(test)]
#[path = "windows_open_tests.rs"]
mod tests;
