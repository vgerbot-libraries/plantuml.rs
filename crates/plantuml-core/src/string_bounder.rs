//! `StringBounder` trait — text measurement abstraction.
//!
//! Ported from: net/sourceforge/plantuml/klimt/font/StringBounder.java
//!
//! Moved to core because it is a foundational abstraction referenced by
//! many crates. The concrete `StringBounderFromWidthTable` implementation
//! lives in `plantuml-klimt`.

use crate::file_format::FileFormat;
use crate::geom::XDimension2D;
use crate::u_font::UFont;

/// Measures the rendered size of text strings.
pub trait StringBounder {
    /// Compute the rendered dimension of `text` in `font`.
    fn calculate_dimension(&self, font: &UFont, text: &str) -> XDimension2D;

    /// The file format this bounder is optimized for.
    fn get_file_format(&self) -> FileFormat;

    /// Optional property probe (used by some backends).
    fn matches_property(&self, _property_name: &str) -> bool {
        false
    }

    /// Descent of `text` in `font`. Default mirrors the Java implementation.
    fn get_descent(&self, font: &UFont, _text: &str) -> f64 {
        font.size_2d() / 4.5
    }
}
