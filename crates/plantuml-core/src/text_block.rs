//! `TextBlock` trait — a drawable block of text/graphics.
//!
//! Ported from: net/sourceforge/plantuml/klimt/shape/TextBlock.java
//! (moved to core as a foundational abstraction).
//!
//! The Java interface references `UGraphic` and `StringBounder` (klimt) and
//! `HorizontalAlignment`. Here we define the minimal trait surface; the
//! drawing method is added once `UGraphic` is available in klimt.

use crate::geom::XDimension2D;

/// A block that can report its dimensions and be drawn.
pub trait TextBlock {
    /// The natural dimensions of the block.
    fn calculate_dimension(&self) -> XDimension2D;
}
