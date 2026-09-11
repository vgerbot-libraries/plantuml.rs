//! A `ReadLine` backed by a `BufRead`.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineReader`.

use std::io::{self, BufRead};

use crate::StringLocated;
use crate::preproc::read_line::ReadLine;
use crate::preproc::read_line_numbered::ReadLineNumbered;
use crate::stubs::LineLocation;

/// A `ReadLine` that reads from a `BufRead` (e.g. a file or byte stream).
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineReader`.
pub struct ReadLineReader {
    reader: Box<dyn BufRead>,
    #[allow(dead_code)]
    location: LineLocation,
    description: String,
    line_number: u32,
}

impl ReadLineReader {
    /// Creates a `ReadLine` from a byte slice and description.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineReader.create(byte[], String)`.
    #[must_use]
    pub fn from_bytes(data: &[u8], description: &str) -> Self {
        let reader = Box::new(io::Cursor::new(data.to_vec()));
        Self {
            reader,
            location: LineLocation::new(Some(description.to_string()), 0),
            description: description.to_string(),
            line_number: 0,
        }
    }

    /// Creates a `ReadLine` from a `BufRead` and description.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.ReadLineReader.create(Reader, String, LineLocation)`.
    #[must_use]
    pub fn new(reader: Box<dyn BufRead>, description: &str, parent: Option<LineLocation>) -> Self {
        let desc = if description.is_empty() { "?" } else { description };
        Self {
            reader,
            location: LineLocation::new(Some(desc.to_string()), parent.map_or(0, |p| p.line)),
            description: desc.to_string(),
            line_number: 0,
        }
    }

    /// Creates a `ReadLine` from a string and description.
    #[must_use]
    pub fn from_string(s: &str, description: &str) -> Self {
        Self::from_bytes(s.as_bytes(), description)
    }
}

impl ReadLine for ReadLineReader {
    fn read_line(&mut self) -> io::Result<Option<StringLocated>> {
        let mut buf = String::new();
        let n = self.reader.read_line(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        // Strip trailing newline
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        // Strip BOM
        if buf.starts_with('\u{FEFF}') {
            buf = buf[3..].to_string();
        }
        // Replace en-dash with hyphen
        buf = buf.replace('\u{2013}', "-");

        self.line_number += 1;
        let location = LineLocation::new(Some(self.description.clone()), self.line_number);
        Ok(Some(StringLocated::new(buf, location)))
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ReadLineNumbered for ReadLineReader {}

impl std::fmt::Display for ReadLineReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReadLineReader({})", self.description)
    }
}
