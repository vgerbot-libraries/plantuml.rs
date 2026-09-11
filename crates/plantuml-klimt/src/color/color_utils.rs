//! Color utilities — SVG color string conversion.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/ColorUtils.java`

use crate::color::color_mapper::ColorMapper;
use crate::color::h_color::HColor;

/// Converts an `HColor` to an SVG color string.
///
/// Ported from: `HColor.toSvg(ColorMapper)`.
#[must_use]
pub fn color_to_svg(color: &HColor, mapper: &ColorMapper) -> String {
    color.to_svg(mapper)
}
