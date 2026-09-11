//! `HColorSimple` — simple RGB color with alpha.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/HColorSimple.java`

use crate::color::color_mapper::ColorMapper;

/// A simple RGB color with optional alpha channel.
///
/// Ported from: `net/sourceforge/plantuml/klimt/color/HColorSimple.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HColorSimple {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl HColorSimple {
    /// Creates a new `HColorSimple` from RGBA components.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new opaque `HColorSimple` from RGB components.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Returns the red component (0-255).
    #[must_use]
    pub const fn r(&self) -> u8 {
        self.r
    }

    /// Returns the green component (0-255).
    #[must_use]
    pub const fn g(&self) -> u8 {
        self.g
    }

    /// Returns the blue component (0-255).
    #[must_use]
    pub const fn b(&self) -> u8 {
        self.b
    }

    /// Returns the alpha component (0-255).
    #[must_use]
    pub const fn a(&self) -> u8 {
        self.a
    }

    /// Converts this color to an SVG color string.
    ///
    /// For opaque colors: `#RRGGBB` (or `#RGB` if shortenable).
    /// For colors with alpha < 255: `#RRGGBBAA`.
    ///
    /// Ported from: `HColorSimple.toSvg(ColorMapper)` via `XColor.toSvg()`.
    #[must_use]
    pub fn to_svg(&self, _mapper: &ColorMapper) -> String {
        if self.a < 255 {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        } else {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        }
    }

    /// Converts this color to an RGB hex string (without `#`).
    ///
    /// Ported from: `HColorSimple.toRGB(ColorMapper)`.
    #[must_use]
    pub fn to_rgb(&self, _mapper: &ColorMapper) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[allow(dead_code)]
/// Shortens a hex color string if possible.
///
/// `#FFFFFF` → `#FFF`, `#E2E2F0` → `#E2E2F0` (not shortenable).
fn shorten_color(hex: &str) -> String {
    if hex.len() == 7 {
        // #RRGGBB
        let chars: Vec<char> = hex.chars().collect();
        if chars[1] == chars[2] && chars[3] == chars[4] && chars[5] == chars[6] {
            return format!("#{}{}{}", chars[1], chars[3], chars[5]);
        }
    }
    hex.to_string()
}
