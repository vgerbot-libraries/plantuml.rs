//! Exact text metrics shared by the mindmap and WBS renderers.
//!
//! Ported from:
//! - `net/sourceforge/plantuml/StringBounderSvg.java`
//! - `net/sourceforge/plantuml/FileFormat.java` (`getJavaDimension`)
//! - `net/sourceforge/plantuml/activitydiagram3/ftile/vertical/FtileBoxOld.java`
//!
//! Both diagrams draw their labels inside an `FtileBoxOld`. The box adds the
//! node-style padding (10 on each side) to the AWT text bounds, i.e. 20 pixels
//! to each dimension. The label baseline sits at `padding_top + ascent` below
use plantuml_core::file_format::FileFormat;
use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;
use plantuml_klimt::string_bounder_svg::StringBounderSvg;

/// Padding inside each box, per side (WBS Padding 10 / mindmap node Padding 10).
pub const BOX_PADDING: f64 = 10.0;

/// AWT ascent (SansSerif, fractional metrics) per point size, measured from the
/// reference JVM. Used to place the text baseline; the SVG cleaner rounds to
/// three decimals, so four decimals here are sufficient.
const ASCENT_12: f64 = 12.8280897;
const ASCENT_14: f64 = 14.9659348;
const DESCENT_12: f64 = 3.5160246;
const DESCENT_14: f64 = 4.1019821;

/// Exact AWT `getStringBounds` height (ascent + descent); this depends only on
/// the font size, not the label. The Rust bounder rounds this to three decimals
/// (16.344), which loses ~0.0001 per box and drifts across a rounding boundary
/// in deeply nested trees, so we restore the full-precision JVM value.
fn text_height(font_size: i32) -> f64 {
    match font_size {
        12 => ASCENT_12 + DESCENT_12,
        14 => ASCENT_14 + DESCENT_14,
        other => f64::from(other) * 1.3620,
    }
}

#[must_use]
pub fn ascent(font_size: i32) -> f64 {
    match font_size {
        12 => ASCENT_12,
        14 => ASCENT_14,
        other => f64::from(other) * 0.9120,
    }
}

/// Measured text bounds (`StringBounderSvg`) for a label at a given size.
#[must_use]
pub fn text_bounds(label: &str, font_size: i32) -> (f64, f64) {
    let bounder = StringBounderSvg::new(FileFormat::Svg);
    let font = UFont::sans_serif(font_size);
    let dim = bounder.calculate_dimension(&font, label);
    (dim.width(), text_height(font_size))
}
/// The outer box size (`FtileBoxOld`) wrapping a label: text bounds plus twice
/// the padding on each dimension.
#[must_use]
pub fn box_size(label: &str, font_size: i32) -> (f64, f64) {
    let (w, h) = text_bounds(label, font_size);
    (w + 2.0 * BOX_PADDING, h + 2.0 * BOX_PADDING)
}

/// Baseline y offset from the box top: top padding plus ascent.
#[must_use]
pub fn baseline_y(font_size: i32) -> f64 {
    BOX_PADDING + ascent(font_size)
}
