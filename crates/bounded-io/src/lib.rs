//! Finite physical input. Callers own paths, limits and authorization.
//! Final-component no-follow is not ancestor confinement or an FS sandbox.
//! Unix and Windows regular input; final-component protection is not a sandbox.
mod errors;
mod files;
mod stream;
#[cfg(windows)]
mod windows;
pub use errors::{InputLimit, InputType};
pub use files::{open_regular_file, read_directory, read_regular_file, read_regular_text};
pub use stream::{read_bounded, read_bounded_retrying};
