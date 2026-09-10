//! UEllipse — ellipse shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/UEllipse.java`

use crate::ushape::UShape;

/// An ellipse centered at (x, y) with x-radius and y-radius.
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/UEllipse.java`
#[derive(Debug, Clone, Copy)]
pub struct UEllipse {
    x: f64,
    y: f64,
    x_radius: f64,
    y_radius: f64,
}

impl UEllipse {
    /// Creates a new `UEllipse` centered at (x, y).
    #[must_use]
    pub const fn new(x: f64, y: f64, x_radius: f64, y_radius: f64) -> Self {
        Self { x, y, x_radius, y_radius }
    }

    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }

    #[must_use]
    pub const fn x_radius(&self) -> f64 {
        self.x_radius
    }

    #[must_use]
    pub const fn y_radius(&self) -> f64 {
        self.y_radius
    }
}

crate::impl_ushape!(UEllipse);
