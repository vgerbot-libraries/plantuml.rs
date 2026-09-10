//! Multilines strategy enum — controls how multi-line command lines are filtered.
//!
//! Ported from: `net/sourceforge/plantuml/command/MultilinesStrategy.java`

/// Controls how lines within a multi-line command are cleaned.
///
/// Ported from: `net/sourceforge/plantuml/command/MultilinesStrategy.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MultilinesStrategy {
    /// Filter out lines starting with a single-quote comment (`'`).
    RemoveStartingQuote,
    /// Keep all lines as-is.
    KeepStartingQuote,
}

impl MultilinesStrategy {
    /// Returns `true` if the given line should be kept based on this strategy.
    ///
    /// Ported from: `MultilinesStrategy.cleanList(List<StringLocated>)` in Java.
    pub fn keep_line(&self, line: &str) -> bool {
        match self {
            Self::RemoveStartingQuote => !line.trim_start().starts_with('\''),
            Self::KeepStartingQuote => true,
        }
    }
}
