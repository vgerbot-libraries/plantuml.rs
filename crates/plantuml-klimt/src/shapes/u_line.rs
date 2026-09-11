//! `ULine` — line shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/ULine.java`


/// A line from (x1, y1) to (x2, y2).
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/ULine.java`
#[derive(Debug, Clone, Copy)]
pub struct ULine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl ULine {
    /// Creates a new `ULine` from (x1, y1) to (x2, y2).
    #[must_use]
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    #[must_use]
    pub const fn x1(&self) -> f64 {
        self.x1
    }

    #[must_use]
    pub const fn y1(&self) -> f64 {
        self.y1
    }

    #[must_use]
    pub const fn x2(&self) -> f64 {
        self.x2
    }

    #[must_use]
    pub const fn y2(&self) -> f64 {
        self.y2
    }
}

crate::impl_ushape!(ULine);
