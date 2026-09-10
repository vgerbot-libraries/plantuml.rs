//! A `ReadLine` backed by a list of strings.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineList`.

use std::io;

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;
use crate::stubs::LineLocation;

/// A `ReadLine` backed by an iterator over a list of strings.
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineList`.
pub struct ReadLineList {
    lines: std::vec::IntoIter<String>,
    location: LineLocation,
}

impl ReadLineList {
    /// Creates a new `ReadLineList` from a list of strings and a location.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineList.ReadLineList`.
    #[must_use]
    pub fn new(definition: Vec<String>, location: LineLocation) -> Self {
        Self {
            lines: definition.into_iter(),
            location,
        }
    }
}

impl ReadLine for ReadLineList {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        match self.lines.next() {
            Some(line) => Ok(Some(StringLocated::new(line, self.location.clone()))),
            None => Ok(None),
        }
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}
