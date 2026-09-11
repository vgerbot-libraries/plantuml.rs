//! `ColorMapper` — maps colors for different rendering modes.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/ColorMapper.java`

/// Color mapping mode (identity, dark, monochrome).
///
/// Ported from: `ColorMapper.java` — `MODE` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMapperMode {
    /// No color transformation.
    Identity,
    /// Dark mode — invert colors for dark backgrounds.
    Dark,
    /// Monochrome — convert to grayscale.
    Monochrome,
}

/// Color mapper that transforms colors based on the rendering mode.
///
/// Ported from: `net/sourceforge/plantuml/klimt/color/ColorMapper.java`
#[derive(Debug, Clone, Copy)]
pub struct ColorMapper {
    mode: ColorMapperMode,
}

impl ColorMapper {
    /// Creates an identity color mapper.
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            mode: ColorMapperMode::Identity,
        }
    }

    /// Creates a color mapper with the given mode.
    #[must_use]
    pub const fn new(mode: ColorMapperMode) -> Self {
        Self { mode }
    }

    /// Returns the mapping mode.
    #[must_use]
    pub const fn mode(&self) -> ColorMapperMode {
        self.mode
    }
}

impl Default for ColorMapper {
    fn default() -> Self {
        Self::identity()
    }
}
