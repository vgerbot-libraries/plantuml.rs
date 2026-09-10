//! Trait for reading lines of text.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLine`.

use std::io;

use crate::StringLocated;

/// Trait for reading lines of text, similar to Java's `ReadLine` (which extends `Closeable`).
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLine`.
pub trait ReadLine {
    /// Reads the next line. Returns `Ok(None)` at end of input.
    fn read_line(&mut self) -> io::Result<Option<StringLocated>>;

    /// Closes the reader, releasing any underlying resources.
    fn close(&mut self) -> io::Result<()>;
}
