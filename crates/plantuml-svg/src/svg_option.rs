//! SvgOption — configuration for SVG output.
//!
//! Ported from: `net/atmp/SvgOption.java`

use indexmap::IndexMap;
use plantuml_klimt::color::{ColorMapper, HColor};

/// Length adjustment mode for text.
///
/// Ported from: `net/sourceforge/plantuml/style/LengthAdjust.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthAdjust {
    None,
    Spacing,
    SpacingAndGlyphs,
}

impl LengthAdjust {
    /// Default value — `Spacing`.
    #[must_use]
    pub const fn default_value() -> Self {
        Self::Spacing
    }
}

/// Configuration for SVG output.
///
/// Ported from: `net/atmp/SvgOption.java`
#[derive(Debug, Clone)]
pub struct SvgOption {
    length_adjust: LengthAdjust,
    preserve_aspect_ratio: String,
    svg_dimension_style: bool,
    min_dim: (f64, f64),
    backcolor: Option<HColor>,
    scale: f64,
    decimal: usize,
    color_mapper: ColorMapper,
    svg_title: Option<String>,
    desc: Option<String>,
    root_attributes: IndexMap<String, String>,
}

impl SvgOption {
    /// Creates a basic `SvgOption` with defaults.
    ///
    /// Ported from: `SvgOption.basic()`.
    #[must_use]
    pub fn basic() -> Self {
        Self {
            length_adjust: LengthAdjust::default_value(),
            preserve_aspect_ratio: "none".to_string(),
            svg_dimension_style: true,
            min_dim: (0.0, 0.0),
            backcolor: None,
            scale: 1.0,
            decimal: 3,
            color_mapper: ColorMapper::identity(),
            svg_title: None,
            desc: None,
            root_attributes: IndexMap::new(),
        }
    }

    /// Returns the length adjust mode.
    #[must_use]
    pub const fn length_adjust(&self) -> LengthAdjust {
        self.length_adjust
    }

    /// Returns the preserveAspectRatio value.
    #[must_use]
    pub fn preserve_aspect_ratio(&self) -> &str {
        &self.preserve_aspect_ratio
    }

    /// Returns whether SVG dimension style is enabled.
    #[must_use]
    pub const fn svg_dimension_style(&self) -> bool {
        self.svg_dimension_style
    }

    /// Returns the minimum dimension (width, height).
    #[must_use]
    pub const fn min_dim(&self) -> (f64, f64) {
        self.min_dim
    }

    /// Returns the background color, if any.
    #[must_use]
    pub fn backcolor(&self) -> Option<&HColor> {
        self.backcolor.as_ref()
    }

    /// Returns the scale factor.
    #[must_use]
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Returns the decimal precision.
    #[must_use]
    pub const fn decimal(&self) -> usize {
        self.decimal
    }

    /// Returns the color mapper.
    #[must_use]
    pub const fn color_mapper(&self) -> &ColorMapper {
        &self.color_mapper
    }

    /// Returns the title, if any.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.svg_title.as_deref()
    }


    /// Returns the description, if any.
    #[must_use]
    pub fn desc(&self) -> Option<&str> {
        self.desc.as_deref()
    }

    /// Sets the SVG title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.svg_title = Some(title.into());
    }

    /// Sets the SVG description.
    pub fn set_desc(&mut self, desc: impl Into<String>) {
        self.desc = Some(desc.into());
    }
    /// Returns the root attributes.
    #[must_use]
    pub fn root_attributes(&self) -> &IndexMap<String, String> {
        &self.root_attributes
    }

    /// Sets a root attribute.
    pub fn set_root_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.root_attributes.insert(key.into(), value.into());
    }

    /// Sets the scale.
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// Sets the background color.
    pub fn set_backcolor(&mut self, color: HColor) {
        self.backcolor = Some(color);
    }
}

impl Default for SvgOption {
    fn default() -> Self {
        Self::basic()
    }
}
