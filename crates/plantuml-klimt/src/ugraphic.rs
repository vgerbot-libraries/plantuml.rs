//! `UGraphic` — the drawing surface abstraction.
//!
//! Ported from: net/sourceforge/plantuml/klimt/drawing/UGraphic.java
//!
//! The Java `UGraphic` is a generic abstract class `UGraphic<O>` with a driver
//! registry keyed by shape class. Concrete backends (`UGraphicSvg`,
//! `UGraphicTxt`, …) subclass it and register their drivers.
//!
//! In Rust we model this as a trait. The full driver-registry mechanics are
//! ported with the first concrete backend (Phase 6, SVG). Here we define the
//! trait surface so downstream crates can depend on it.

use plantuml_core::string_bounder::StringBounder;
use plantuml_core::u_font::UFont;

use crate::uchange::UChange;
use crate::ushape::UShape;

/// A drawing surface that accepts shapes and state changes.
pub trait UGraphic {
    /// The change context type (color, stroke, font, translate, …).
    type Change: UChange;

    /// Draw a shape onto this surface with the current state.
    fn draw(&mut self, shape: &dyn UShape);

    /// Apply a state change, returning a new (or mutated) surface.
    fn apply(&mut self, change: Self::Change);

    /// The string bounder used for text measurement.
    fn get_string_bounder(&self) -> &dyn StringBounder;

    /// The current font.
    fn get_font(&self) -> &UFont;
}
