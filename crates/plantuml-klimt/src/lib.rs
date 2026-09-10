//! 2D graphics abstraction layer.
//!
//! Ported from: net/sourceforge/plantuml/klimt/ (drawing, shape, font, color).
//!
//! Phase 1 ports the deterministic text-measurement path (width table +
//! `StringBounderFromWidthTable`) and the `UGraphic`/`UShape`/`UChange`/
//! `UDriver` trait surface. Shapes, colors, and the SVG/TXT drivers are
//! added in later phases as their backends are ported.

pub mod changes;
pub mod color;
pub mod shapes;
pub mod string_bounder_from_width_table;
pub mod unicode_block;
pub mod unicode_font_width_sans_serif;
pub mod uchange;
pub mod udriver;
pub mod ugraphic;
pub mod ushape;

pub use changes::{UChangeColor, UChangeFont, UChangeStroke, UClip, UTranslate};
pub use color::{ColorMapper, ColorMapperMode, HColor, HColorSet, HColorSimple, HColors};
pub use shapes::{UEllipse, ULine, UPath, UPolygon, URectangle, UText};
pub use string_bounder_from_width_table::StringBounderFromWidthTable;
pub use uchange::UChange;
pub use udriver::UDriver;
pub use ugraphic::UGraphic;
pub use ushape::UShape;
