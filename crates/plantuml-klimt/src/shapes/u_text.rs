//! `UText` — text shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/UText.java`

use plantuml_core::u_font::UFont;

/// A text string with position, font, and text length.
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/UText.java`
#[derive(Debug, Clone)]
pub struct UText {
    text: String,
    x: f64,
    y: f64,
    font: UFont,
    text_length: f64,
}

impl UText {
    /// Creates a new `UText` at (x, y) with the given font.
    #[must_use]
    pub fn new(text: impl Into<String>, x: f64, y: f64, font: UFont, text_length: f64) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            font,
            text_length,
        }
    }

    /// Returns the text content.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn font(&self) -> &UFont {
        &self.font
    }

    #[must_use]
    pub const fn text_length(&self) -> f64 {
        self.text_length
    }
}

crate::impl_ushape!(UText);
