//! Concrete `UChange` implementations — color, stroke, font, translate, clip.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/drawing/` package
use crate::HColor;
use crate::uchange::UChange;
use plantuml_core::u_font::UFont;

/// Changes the current drawing color (stroke/fill).
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/UChangeColor.java`
#[derive(Debug, Clone)]
pub struct UChangeColor {
    color: HColor,
}

impl UChangeColor {
    #[must_use]
    pub const fn new(color: HColor) -> Self {
        Self { color }
    }

    #[must_use]
    pub const fn color(&self) -> &HColor {
        &self.color
    }
}

impl UChange for UChangeColor {}

/// Changes the stroke width and dash array.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/UChangeStroke.java`
#[derive(Debug, Clone)]
pub struct UChangeStroke {
    stroke_width: f64,
    dash_array: Option<[f64; 2]>,
}

impl UChangeStroke {
    #[must_use]
    pub const fn new(stroke_width: f64) -> Self {
        Self {
            stroke_width,
            dash_array: None,
        }
    }

    #[must_use]
    pub const fn with_dash(stroke_width: f64, dash: [f64; 2]) -> Self {
        Self {
            stroke_width,
            dash_array: Some(dash),
        }
    }

    #[must_use]
    pub const fn stroke_width(&self) -> f64 {
        self.stroke_width
    }

    #[must_use]
    pub const fn dash_array(&self) -> Option<[f64; 2]> {
        self.dash_array
    }
}

impl UChange for UChangeStroke {}

/// Changes the current font.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/UChangeFont.java`
#[derive(Debug, Clone)]
pub struct UChangeFont {
    font: UFont,
}

impl UChangeFont {
    #[must_use]
    pub const fn new(font: UFont) -> Self {
        Self { font }
    }

    #[must_use]
    pub const fn font(&self) -> &UFont {
        &self.font
    }
}

impl UChange for UChangeFont {}

/// Translates the drawing position by (dx, dy).
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/UTranslate.java`
#[derive(Debug, Clone, Copy)]
pub struct UTranslate {
    dx: f64,
    dy: f64,
}

impl UTranslate {
    #[must_use]
    pub const fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    #[must_use]
    pub const fn dx(&self) -> f64 {
        self.dx
    }

    #[must_use]
    pub const fn dy(&self) -> f64 {
        self.dy
    }

    /// Returns the combined translation of self and other.
    #[must_use]
    pub const fn compose(&self, other: &Self) -> Self {
        Self::new(self.dx + other.dx, self.dy + other.dy)
    }
}

impl UChange for UTranslate {}

/// Clips the drawing area to a rectangle.
///
/// Ported from: `net/sourceforge/plantuml/klimt/drawing/UClip.java`
#[derive(Debug, Clone, Copy)]
pub struct UClip {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl UClip {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
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
}

impl UChange for UClip {}
