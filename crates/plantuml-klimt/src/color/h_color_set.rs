//! `HColorSet` — named color registry.
//!
//! Ported from: `net/sourceforge/plantuml/klimt/color/HColorSet.java`

use crate::color::h_color::HColor;
use crate::color::h_color_simple::HColorSimple;
use crate::color::h_colors::HColors;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Registry of named colors.
///
/// Ported from: `net/sourceforge/plantuml/klimt/color/HColorSet.java`
pub struct HColorSet;

static NAMED_COLORS: LazyLock<HashMap<&'static str, HColor>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Standard CSS/X11 colors used by PlantUML
    m.insert("black", HColor::Simple(HColorSimple::rgb(0, 0, 0)));
    m.insert("white", HColor::Simple(HColorSimple::rgb(255, 255, 255)));
    m.insert("red", HColor::Simple(HColorSimple::rgb(255, 0, 0)));
    m.insert("green", HColor::Simple(HColorSimple::rgb(0, 128, 0)));
    m.insert("blue", HColor::Simple(HColorSimple::rgb(0, 0, 255)));
    m.insert("yellow", HColor::Simple(HColorSimple::rgb(255, 255, 0)));
    m.insert("orange", HColor::Simple(HColorSimple::rgb(255, 165, 0)));
    m.insert("purple", HColor::Simple(HColorSimple::rgb(128, 0, 128)));
    m.insert("pink", HColor::Simple(HColorSimple::rgb(255, 192, 203)));
    m.insert("gray", HColor::Simple(HColorSimple::rgb(128, 128, 128)));
    m.insert("grey", HColor::Simple(HColorSimple::rgb(128, 128, 128)));
    m.insert("silver", HColor::Simple(HColorSimple::rgb(192, 192, 192)));
    m.insert("transparent", HColor::Transparent);
    m
});

impl HColorSet {
    /// Gets a color by name.
    ///
    /// Ported from: `HColorSet.getColorOrWhite(String)`.
    #[must_use]
    pub fn get_color_or_white(name: &str) -> HColor {
        Self::get_color(name).unwrap_or_else(HColors::white)
    }

    /// Gets a color by name, returning `None` if not found.
    ///
    /// Ported from: `HColorSet.getColor(String)`.
    #[must_use]
    pub fn get_color(name: &str) -> Option<HColor> {
        let lower = name.to_lowercase();
        NAMED_COLORS.get(lower.as_str()).cloned()
    }

    /// Returns `true` if a color with the given name exists.
    #[must_use]
    pub fn contains(name: &str) -> bool {
        NAMED_COLORS.contains_key(name.to_lowercase().as_str())
    }
}
