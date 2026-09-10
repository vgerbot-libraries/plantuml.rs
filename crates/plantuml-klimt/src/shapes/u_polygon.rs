//! UPolygon — polygon shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/UPolygon.java`

use crate::ushape::UShape;

/// A polygon defined by a list of points.
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/UPolygon.java`
#[derive(Debug, Clone)]
pub struct UPolygon {
    points: Vec<(f64, f64)>,
}

impl UPolygon {
    /// Creates a new empty `UPolygon`.
    #[must_use]
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Creates a `UPolygon` from a list of (x, y) points.
    #[must_use]
    pub fn from_points(points: Vec<(f64, f64)>) -> Self {
        Self { points }
    }

    /// Adds a point to the polygon.
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    /// Returns the points as a flat list [x1, y1, x2, y2, ...].
    #[must_use]
    pub fn flat_points(&self) -> Vec<f64> {
        let mut result = Vec::with_capacity(self.points.len() * 2);
        for (x, y) in &self.points {
            result.push(*x);
            result.push(*y);
        }
        result
    }

    /// Returns the points.
    #[must_use]
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }
}

impl Default for UPolygon {
    fn default() -> Self {
        Self::new()
    }
}

crate::impl_ushape!(UPolygon);
