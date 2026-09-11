//! `URectangle` — rectangle shape.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/shape/URectangle.java`


/// A rectangle shape with position, dimensions, and corner radius.
///
/// Ported from: `net/sourceforge/plantuml/klimt/shape/URectangle.java`
#[derive(Debug, Clone)]
pub struct URectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rx: f64,
    ry: f64,
}

impl URectangle {
    /// Creates a new `URectangle` at (x, y) with the given width and height.
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rx: 0.0,
            ry: 0.0,
        }
    }

    /// Creates a new `URectangle` with corner radius.
    #[must_use]
    pub const fn with_corners(x: f64, y: f64, width: f64, height: f64, rx: f64, ry: f64) -> Self {
        Self { x, y, width, height, rx, ry }
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
    pub const fn width(&self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> f64 {
        self.height
    }

    #[must_use]
    pub const fn rx(&self) -> f64 {
        self.rx
    }

    #[must_use]
    pub const fn ry(&self) -> f64 {
        self.ry
    }
}

crate::impl_ushape!(URectangle);
