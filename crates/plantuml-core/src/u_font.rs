//! `UFont` — font descriptor (family, size, style).
//!
//! Ported from: net/sourceforge/plantuml/klimt/font/UFont.java
//!              net/sourceforge/plantuml/klimt/font/UFontContext.java
//!
//! The Java `UFont` stores a `FontStack` + `UFontFace` (CSS weight/italic axis)
//! and resolves to an AWT `Font` at draw time. Rust has no AWT; for the
//! deterministic width-table bounder we only need the family name, size, and
//! a style flag. The full `UFontFace`/`FontStack` machinery is ported in a
//! later phase when style support is needed.

/// Rendering context in which a font family name is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UFontContext {
    Eps,
    Svg,
    G2d,
    Tikz,
    Pdf,
}

/// Font style (binary bold/italic, matching AWT's `Font` constants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FontStyle {
    pub bold: bool,
    pub italic: bool,
}

impl FontStyle {
    #[must_use]
    pub const fn plain() -> Self {
        Self { bold: false, italic: false }
    }

    #[must_use]
    pub const fn bold() -> Self {
        Self { bold: true, italic: false }
    }

    #[must_use]
    pub const fn italic() -> Self {
        Self { bold: false, italic: true }
    }

    #[must_use]
    pub const fn bold_italic() -> Self {
        Self { bold: true, italic: true }
    }
}

/// A font descriptor: family name, size, and style.
#[derive(Debug, Clone, PartialEq)]
pub struct UFont {
    family: String,
    style: FontStyle,
    size: i32,
}

impl UFont {
    #[must_use]
    pub fn new(family: impl Into<String>, style: FontStyle, size: i32) -> Self {
        Self { family: family.into(), style, size }
    }

    #[must_use]
    pub fn sans_serif(size: i32) -> Self {
        Self::new("sans-serif", FontStyle::plain(), size)
    }

    /// The font family name, resolved for the given context.
    ///
    /// The Java version switches on `UFontContext` and may return a
    /// PostScript name or an SVG family. For the deterministic path the
    /// family is returned unchanged.
    #[must_use]
    pub fn family(&self, _text: &str, _context: UFontContext) -> &str {
        &self.family
    }

    #[must_use]
    pub const fn style(&self) -> FontStyle {
        self.style
    }

    #[must_use]
    pub const fn size(&self) -> i32 {
        self.size
    }

    #[must_use]
    pub fn size_2d(&self) -> f64 {
        f64::from(self.size)
    }

    #[must_use]
    pub fn with_size(&self, size: f64) -> Self {
        Self { family: self.family.clone(), style: self.style, size: size as i32 }
    }

    #[must_use]
    pub fn with_style(&self, style: FontStyle) -> Self {
        Self { family: self.family.clone(), style, size: self.size }
    }
}

impl std::fmt::Display for UFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.family, self.style.bold, self.style.italic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_basics() {
        let f = UFont::sans_serif(16);
        assert_eq!(f.family("", UFontContext::Svg), "sans-serif");
        assert!((f.size_2d() - 16.0).abs() < 1e-9);
        assert_eq!(f.style(), FontStyle::plain());
    }

    #[test]
    fn with_size() {
        let f = UFont::sans_serif(16).with_size(24.0);
        assert_eq!(f.size(), 24);
    }
}
