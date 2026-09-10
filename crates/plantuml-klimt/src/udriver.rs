//! `UDriver` — backend-specific shape renderer.
//!
//! Ported from: net/sourceforge/plantuml/klimt/drawing/UDriver.java
//!
//! Each backend (SVG, TXT, PDF, …) provides a `UDriver<T>` per shape type,
//! registered in the `UGraphic` driver map. `draw` renders `shape` onto the
//! backend's drawing surface `g` using the current color mapper.

use plantuml_core::u_font::UFont;

use crate::uchange::UChange;
use crate::ushape::UShape;

/// A driver that knows how to draw one shape type onto a backend surface.
///
/// `G` is the backend's drawing surface (e.g. `SvgGraphics`), `C` is the
/// color/change context carried by the `UGraphic`.
pub trait UDriver<G, C: UChange> {
    /// Draw `shape` onto `g`, applying the current change context `change`.
    fn draw(&self, shape: &dyn UShape, g: &mut G, change: &C);
}


/// Font context carried through `UGraphic` draw calls. Placeholder until
/// the full font/style system is ported.
#[derive(Debug, Clone)]
pub struct FontContext {
    font: UFont,
}

impl FontContext {
    #[must_use]
    pub const fn new(font: UFont) -> Self {
        Self { font }
    }

    #[must_use]
    pub const fn font(&self) -> &UFont {
        &self.font
    }
}
