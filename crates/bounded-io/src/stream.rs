// Derived from Hatter bounded intake; caller budget, no duration/token policy.
use crate::errors::exceeded;
use std::io::{self, Read};
/// Probe one byte beyond the budget without appending or reserving it.
pub fn read_bounded(mut reader: impl Read, limit: u64) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let remaining = limit.saturating_sub(bytes.len() as u64);
        let size = remaining.saturating_add(1).min(chunk.len() as u64) as usize;
        let count = reader.read(&mut chunk[..size])?;
        if count == 0 {
            return Ok(bytes);
        }
        if count as u64 > remaining {
            return Err(exceeded("bytes", limit, limit.saturating_add(1)));
        }
        bytes
            .try_reserve_exact(count)
            .map_err(|_| io::Error::new(io::ErrorKind::OutOfMemory, "input allocation rejected"))?;
        bytes.extend_from_slice(&chunk[..count]);
    }
}

/// Match std Read::read_to_end interruption handling without changing the byte budget.
pub fn read_bounded_retrying(reader: impl Read, limit: u64) -> io::Result<Vec<u8>> {
    struct Retry<R>(R);
    impl<R: Read> Read for Retry<R> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            loop {
                match self.0.read(output) {
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    result => return result,
                }
            }
        }
    }
    read_bounded(Retry(reader), limit)
}
