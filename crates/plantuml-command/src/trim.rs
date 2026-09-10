//! Trim enum — controls how line text is trimmed before matching.
//!
//! Ported from: `net/sourceforge/plantuml/command/Trim.java`

/// Controls how input line text is trimmed before regex matching.
///
/// Ported from: `net/sourceforge/plantuml/command/Trim.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trim {
    /// Trim both leading and trailing whitespace.
    Both,
    /// Trim only leading whitespace.
    LeftOnly,
    /// No trimming.
    None,
}

impl Trim {
    /// Trims the given text according to this trim mode.
    ///
    /// Ported from: `Trim.trim(StringLocated)` in Java.
    #[must_use]
    pub fn trim(&self, s: &str) -> String {
        match self {
            Self::Both => s.trim().to_string(),
            Self::LeftOnly => s.trim_start().to_string(),
            Self::None => s.to_string(),
        }
    }
}
