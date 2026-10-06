// Derived from Hatter bounded intake; original Apache-2.0 LICENSE/NOTICE retained.
use std::{fmt, io};
#[derive(Debug)]
pub struct InputLimit {
    pub dimension: &'static str,
    pub limit: u64,
    pub observed_at_least: u64,
}
impl fmt::Display for InputLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "input {} limit {} exceeded (at least {})",
            self.dimension, self.limit, self.observed_at_least
        )
    }
}
impl std::error::Error for InputLimit {}
#[derive(Debug)]
pub struct InputType(pub(crate) &'static str);
impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "regular input {} required", self.0)
    }
}
impl std::error::Error for InputType {}
pub(crate) fn exceeded(dimension: &'static str, limit: u64, observed: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        InputLimit {
            dimension,
            limit,
            observed_at_least: observed,
        },
    )
}
pub(crate) fn wrong_type(expected: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, InputType(expected))
}
