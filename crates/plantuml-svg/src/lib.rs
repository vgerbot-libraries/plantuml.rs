//! PlantUML SVG rendering backend.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/svg/` package.

pub mod svg_graphics;
pub mod svg_option;
pub mod ugraphic_svg;
pub mod xml;

pub use svg_graphics::{SvgGraphics, TransparentFillBehavior};
pub use svg_option::{LengthAdjust, SvgOption};
pub use ugraphic_svg::{SvgChange, UGraphicSvg};
pub use xml::{XmlDocument, XmlLeaf, XmlNode, XmlWriter};
