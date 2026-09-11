//! Color types — `HColor` and color utilities.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/` package

mod color_mapper;
mod color_utils;
mod h_color;
mod h_color_set;
mod h_color_simple;
mod h_colors;

pub use color_mapper::{ColorMapper, ColorMapperMode};
pub use color_utils::color_to_svg;
pub use h_color::{HColor, TransparentFillBehavior};
pub use h_color_set::HColorSet;
pub use h_color_simple::HColorSimple;
pub use h_colors::HColors;
