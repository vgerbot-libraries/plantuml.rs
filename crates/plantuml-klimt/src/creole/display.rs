//! `Display` — multi-line rich text display.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/creole/Display.java`
//!
//! A `Display` holds an ordered list of text lines. In the Java original,
//! each line is a `CharSequence` with Creole markup support. This Rust
//! port stores lines as `String` values; Creole parsing will be added
//! when the full rendering pipeline is ported.
//!
//! The `Display::NULL` constant represents the null-display (empty,
//! `is_null() == true`), following the Null Object Pattern from the
//! Java original.

use std::slice::Iter;

/// Multi-line rich text display.
///
/// Ported from: `net/sourceforge/plantuml/klimt/creole/Display.java`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Display {
    /// The display lines. An empty vec means the null display.
    lines: Vec<String>,
    /// Whether this is the explicit null display (`Display::NULL`).
    is_null: bool,
}

impl Display {
    /// The null display — empty and `is_null() == true`.
    ///
    /// Ported from: `Display.NULL`.
    pub const NULL: Self = Self {
        lines: Vec::new(),
        is_null: true,
    };

    /// Creates a `Display` from a single text line.
    ///
    /// Ported from: `Display.create(CharSequence)`.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        if text.is_empty() {
            return Self::empty();
        }
        Self {
            lines: vec![text],
            is_null: false,
        }
    }

    /// Creates a `Display` from multiple text lines.
    ///
    /// Ported from: `Display.create(List<CharSequence>)`.
    #[must_use]
    pub fn from_lines(lines: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let lines: Vec<String> = lines.into_iter().map(Into::into).collect();
        if lines.is_empty() {
            return Self::empty();
        }
        Self {
            lines,
            is_null: false,
        }
    }

    /// Creates an empty (but non-null) display.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            lines: Vec::new(),
            is_null: false,
        }
    }

    /// Returns `true` if this is the null display.
    ///
    /// Ported from: `Display.isNull(Display)`.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.is_null
    }

    /// Returns `true` if the display is empty or all whitespace.
    ///
    /// Ported from: `Display.isWhite()`.
    #[must_use]
    pub fn is_white(&self) -> bool {
        self.lines.iter().all(|l| l.trim().is_empty())
    }

    /// Returns the number of lines.
    ///
    /// Ported from: `Display.size()`.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.lines.len()
    }

    /// Returns `true` if there are no lines.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the line at index `i`.
    ///
    /// Ported from: `Display.get(i)`.
    #[must_use]
    pub fn get(&self, i: usize) -> Option<&str> {
        self.lines.get(i).map(String::as_str)
    }

    /// Returns an iterator over the lines.
    ///
    /// Ported from: `Display.iterator()`.
    pub fn iter(&self) -> Iter<'_, String> {
        self.lines.iter()
    }

    /// Returns the lines as a slice.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Appends a line to this display.
    ///
    /// Ported from: `Display.add(CharSequence)`.
    pub fn add(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.is_null = false;
    }

    /// Creates a display from a single string, splitting on newlines.
    ///
    /// Ported from: `Display.createWithNewlines(CharSequence)`.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        if lines.is_empty() {
            return Self::empty();
        }
        Self {
            lines,
            is_null: false,
        }
    }

    /// Static helper: returns `true` if the given display is null.
    ///
    /// Ported from: `Display.isNull(Display)`.
    #[must_use]
    pub const fn is_null_display(display: &Self) -> bool {
        display.is_null
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            f.write_str(line)?;
        }
        Ok(())
    }
}

impl From<String> for Display {
    fn from(s: String) -> Self {
        Self::from_text(&s)
    }
}

impl From<&str> for Display {
    fn from(s: &str) -> Self {
        Self::from_text(s)
    }
}

impl<'a> IntoIterator for &'a Display {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_display() {
        assert!(Display::NULL.is_null());
        assert!(Display::NULL.is_empty());
        assert_eq!(Display::NULL.size(), 0);
    }

    #[test]
    fn test_single_line() {
        let d = Display::new("Hello");
        assert!(!d.is_null());
        assert_eq!(d.size(), 1);
        assert_eq!(d.get(0), Some("Hello"));
    }

    #[test]
    fn test_multi_line() {
        let d = Display::from_lines(["Line 1", "Line 2", "Line 3"]);
        assert_eq!(d.size(), 3);
        assert_eq!(d.get(0), Some("Line 1"));
        assert_eq!(d.get(2), Some("Line 3"));
    }

    #[test]
    fn test_from_text_splits_newlines() {
        let d = Display::from_text("A\nB\nC");
        assert_eq!(d.size(), 3);
        assert_eq!(d.get(1), Some("B"));
    }

    #[test]
    fn test_is_white() {
        assert!(Display::from_lines(["  ", "\t", ""]).is_white());
        assert!(!Display::from_lines(["  ", "x"]).is_white());
    }

    #[test]
    fn test_display_format() {
        let d = Display::from_lines(["A", "B"]);
        assert_eq!(format!("{d}"), "A\nB");
    }
}
