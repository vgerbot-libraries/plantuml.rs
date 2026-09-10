//! LengthAdjust — SVG `textLength`/`lengthAdjust` attribute values.
//!
//! Ported from: `net/sourceforge/plantuml/style/LengthAdjust.java`

/// Value for the SVG `lengthAdjust` attribute.
///
/// Ported from: `net/sourceforge/plantuml/style/LengthAdjust.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthAdjust {
    None,
    Spacing,
    SpacingAndGlyphs,
}

impl LengthAdjust {
    /// The default value.
    #[must_use]
    pub fn default_value() -> Self {
        Self::Spacing
    }
}

impl std::fmt::Display for LengthAdjust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Spacing => write!(f, "spacing"),
            Self::SpacingAndGlyphs => write!(f, "spacingAndGlyphs"),
        }
    }
}
