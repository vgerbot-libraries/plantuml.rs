//! A `ReadLine` that concatenates multiple `ReadLine`s.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineConcat`.

use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;

/// A `ReadLine` that reads from multiple `ReadLine`s in sequence.
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineConcat`.
pub struct ReadLineConcat {
    readers: Vec<Box<dyn ReadLine>>,
    current: usize,
}

impl ReadLineConcat {
    /// Creates a new `ReadLineConcat` from a list of `ReadLine`s.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineConcat.ReadLineConcat`.
    #[must_use]
    pub fn new(readers: Vec<Box<dyn ReadLine>>) -> Self {
        Self { readers, current: 0 }
    }
}

impl ReadLine for ReadLineConcat {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        while self.current < self.readers.len() {
            let result = self.readers[self.current].read_line()?;
            if result.is_some() {
                return Ok(result);
            }
            self.current += 1;
        }
        Ok(None)
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}
