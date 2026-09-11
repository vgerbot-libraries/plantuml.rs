//! A `ReadLine` that also tracks line numbers.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ReadLineNumbered`.


use crate::preproc::read_line::ReadLine;

/// Trait for `ReadLine`s that track line numbers.
///
/// In Java, this is an empty sub-interface of `ReadLine`.
///
/// Ported from `net.sourceforge.plantuml.preproc.ReadLineNumbered`.
pub trait ReadLineNumbered: ReadLine {
    // Marker trait — no additional methods needed.
    // In Java, this was: `public interface ReadLineNumbered extends ReadLine {}`
}

// ReadLineReader implements ReadLineNumbered.
// See `read_line_reader.rs`.
