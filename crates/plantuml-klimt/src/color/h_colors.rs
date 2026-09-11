//! `HColors` — factory for common colors.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/HColors.java`

use crate::color::h_color::HColor;
use crate::color::h_color_simple::HColorSimple;

/// Factory for common named colors.
///
/// Ported from: `net/sourceforge/plantuml/klimt/color/HColors.java`
pub struct HColors;

impl HColors {
    /// Black color.
    #[must_use]
    pub const fn black() -> HColor {
        HColor::Simple(HColorSimple::rgb(0, 0, 0))
    }

    /// White color.
    #[must_use]
    pub const fn white() -> HColor {
        HColor::Simple(HColorSimple::rgb(255, 255, 255))
    }

    /// Transparent color.
    #[must_use]
    pub const fn transparent() -> HColor {
        HColor::Transparent
    }

    /// Red color.
    #[must_use]
    pub const fn red() -> HColor {
        HColor::Simple(HColorSimple::rgb(255, 0, 0))
    }

    /// Green color.
    #[must_use]
    pub const fn green() -> HColor {
        HColor::Simple(HColorSimple::rgb(0, 128, 0))
    }

    /// Blue color.
    #[must_use]
    pub const fn blue() -> HColor {
        HColor::Simple(HColorSimple::rgb(0, 0, 255))
    }

    /// Light gray color (#E2E2F0 — used in sequence diagram headers).
    #[must_use]
    pub const fn light_gray() -> HColor {
        HColor::Simple(HColorSimple::rgb(0xE2, 0xE2, 0xF0))
    }

    /// Dark gray color (#181818 — used for strokes).
    #[must_use]
    pub const fn dark_gray() -> HColor {
        HColor::Simple(HColorSimple::rgb(0x18, 0x18, 0x18))
    }

    /// Creates a color from a hex string like `#E2E2F0` or `#FFF`.
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<HColor> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(HColor::Simple(HColorSimple::rgb(r, g, b)))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(HColor::Simple(HColorSimple::rgb(r, g, b)))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(HColor::Simple(HColorSimple::new(r, g, b, a)))
            }
            _ => None,
        }
    }
}
