//! A lazily-loaded SVG sprite from the stdlib.
//!
//! Ported from `net.sourceforge.plantuml.preproc.StdlibSpriteSvg`.

use crate::stubs::Sprite;

/// An SVG sprite that is lazily loaded from the stdlib.
///
/// Ported from `net.sourceforge.plantuml.preproc.StdlibSpriteSvg`.
pub struct StdlibSpriteSvg {
    svg_data: Option<String>,
}

impl StdlibSpriteSvg {
    /// Creates a new `StdlibSpriteSvg` with the given SVG data.
    ///
    /// In Java, this takes a `Stdlib` and a name, and lazily reads the SVG sprite.
    /// Here we store the SVG data directly.
    #[must_use]
    pub fn new(svg_data: String) -> Self {
        Self {
            svg_data: Some(svg_data),
        }
    }

    /// Creates a new `StdlibSpriteSvg` that will lazily load from the given stdlib.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.StdlibSpriteSvg.StdlibSpriteSvg`.
    #[must_use]
    pub fn from_stdlib(_lib: &crate::preproc::stdlib::Stdlib, _name: &str) -> Self {
        // In the full implementation, this would lazily call lib.read_svg_sprite(name).
        Self { svg_data: None }
    }
}

impl Sprite for StdlibSpriteSvg {
    fn as_text_block(&self) -> String {
        // In the full implementation, this delegates to the loaded Sprite.
        self.svg_data.as_deref().unwrap_or("").to_string()
    }
}
