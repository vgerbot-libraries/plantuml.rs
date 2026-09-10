//! A `ReadLine` that yields a single line.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineSimple`.

use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;

/// A `ReadLine` that returns a single `StringLocated` (with an optional error),
/// then returns `None` on subsequent calls.
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineSimple`.
pub struct ReadLineSimple {
    data: StringLocated,
    error: String,
    current: usize,
}

impl ReadLineSimple {
    /// Creates a new `ReadLineSimple` with the given data and error message.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineSimple.ReadLineSimple`.
    #[must_use]
    pub fn new(s: StringLocated, error: impl Into<String>) -> Self {
        Self {
            data: s,
            error: error.into(),
            current: 0,
        }
    }
}

impl ReadLine for ReadLineSimple {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        if self.current > 0 {
            return Ok(None);
        }
        self.current += 1;
        Ok(Some(self.data.with_error_preprocessor(&self.error)))
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}
