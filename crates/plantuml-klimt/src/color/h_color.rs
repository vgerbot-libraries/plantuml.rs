//! HColor — abstract color type.
//!
use crate::color::h_color_simple::HColorSimple;

use crate::color::color_mapper::ColorMapper;

/// How transparent fills are rendered.
///
/// Ported from: `HColor.TransparentFillBehavior`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentFillBehavior {
    /// Use `fill:none`.
    WithFillNone,
    /// Use `fill-opacity:0`.
    WithFillOpacity,
}

/// Abstract color type — the base for all PlantUML colors.
///
/// In Java this is an abstract class with subclasses for simple RGB colors,
/// gradients, named colors, etc. In Rust we use an enum to keep things
/// simple and deterministic.
///
/// Ported from: `net/sourceforge/plantuml/klimt/color/HColor.java`
#[derive(Debug, Clone, PartialEq)]
pub enum HColor {
    /// Simple RGB color with optional alpha.
    Simple(HColorSimple),
    /// Fully transparent.
    Transparent,
    /// Named color (resolved lazily via HColorSet).
    Named(String),
}

impl HColor {
    /// Returns `true` if this color is fully transparent.
    ///
    /// Ported from: `HColor.isTransparent()`.
    #[must_use]
    pub fn is_transparent(&self) -> bool {
        matches!(self, Self::Transparent)
    }

    /// Converts this color to an SVG color string (e.g. `#E2E2F0`, `#000`).
    ///
    /// Ported from: `HColor.toSvg(ColorMapper)`.
    #[must_use]
    pub fn to_svg(&self, mapper: &ColorMapper) -> String {
        match self {
            Self::Transparent => "#00000000".to_string(),
            Self::Simple(s) => s.to_svg(mapper),
            Self::Named(name) => {
                // Resolve named color via HColorSet
                if let Some(resolved) = HColorSet::get_color(name) {
                    resolved.to_svg(mapper)
                } else {
                    format!("#{name}")
                }
            }
        }
    }

    /// Converts this color to an RGB hex string (e.g. `E2E2F0`).
    ///
    /// Ported from: `HColor.toRGB(ColorMapper)`.
    #[must_use]
    pub fn to_rgb(&self, mapper: &ColorMapper) -> String {
        match self {
            Self::Transparent => "000000".to_string(),
            Self::Simple(s) => s.to_rgb(mapper),
            Self::Named(name) => {
                if let Some(resolved) = HColorSet::get_color(name) {
                    resolved.to_rgb(mapper)
                } else {
                    name.clone()
                }
            }
        }
    }

    /// Creates a simple RGB color from red, green, blue components (0-255).
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Simple(HColorSimple::new(r, g, b, 255))
    }

    /// Creates a simple RGBA color from red, green, blue, alpha components.
    #[must_use]
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::Simple(HColorSimple::new(r, g, b, a))
    }
}

use crate::color::h_color_set::HColorSet;

/// Marker trait for background changes.
///
/// Ported from: `net/sourceforge/plantuml/klimt/UBackground.java`
pub trait UBackground: crate::uchange::UChange {
    fn get_back_color(&self) -> HColor;
}

/// A background change carrying an HColor.
#[derive(Debug, Clone)]
pub struct Back {
    color: HColor,
}

impl Back {
    #[must_use]
    pub fn new(color: HColor) -> Self {
        Self { color }
    }

    #[must_use]
    pub fn color(&self) -> &HColor {
        &self.color
    }
}

impl UBackground for Back {
    fn get_back_color(&self) -> HColor {
        self.color.clone()
    }
}

impl crate::uchange::UChange for Back {}
