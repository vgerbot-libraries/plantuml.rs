//! A `FileFormat` with rendering parameters.
//!
//! Ported from: net/sourceforge/plantuml/FileFormatOption.java
//!
//! The Java class carries many AWT/Tikz-specific fields. Here we keep the
//! portable subset (`file_format`, `with_metadata`, `scale`, `decimal`,
//! `use_red_for_error`, `preserve_aspect_ratio`, `watermark`, `svg_link_target`,
//! `hover_color`). Color-mapper and Tikz distortion are added when their
//! crates are ported.

use crate::file_format::FileFormat;

/// A `FileFormat` plus rendering options.
#[derive(Debug, Clone)]
pub struct FileFormatOption {
    file_format: FileFormat,
    with_metadata: bool,
    use_red_for_error: bool,
    svg_link_target: Option<String>,
    hover_color: Option<String>,
    scale: f64,
    preserve_aspect_ratio: Option<String>,
    watermark: Option<String>,
    decimal: i32,
}

impl FileFormatOption {
    #[must_use]
    pub fn new(file_format: FileFormat) -> Self {
        Self {
            file_format,
            with_metadata: true,
            use_red_for_error: false,
            svg_link_target: None,
            hover_color: None,
            scale: 1.0,
            preserve_aspect_ratio: None,
            watermark: None,
            decimal: -1,
        }
    }

    #[must_use]
    pub fn new_with_metadata(file_format: FileFormat, with_metadata: bool) -> Self {
        let mut o = Self::new(file_format);
        o.with_metadata = with_metadata;
        o
    }

    #[must_use]
    pub const fn file_format(&self) -> FileFormat {
        self.file_format
    }

    #[must_use]
    pub const fn is_with_metadata(&self) -> bool {
        self.with_metadata
    }

    pub fn hide_metadata(&mut self) {
        self.with_metadata = false;
    }

    #[must_use]
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    #[must_use]
    pub fn with_scale(&self, scale: f64) -> Self {
        let mut o = self.clone();
        o.scale = scale;
        o
    }

    #[must_use]
    pub const fn decimal(&self) -> i32 {
        self.decimal
    }

    #[must_use]
    pub fn with_decimal(&self, decimal: i32) -> Self {
        let mut o = self.clone();
        o.decimal = decimal;
        o
    }

    #[must_use]
    pub const fn is_use_red_for_error(&self) -> bool {
        self.use_red_for_error
    }

    #[must_use]
    pub fn with_use_red_for_error(&self) -> Self {
        let mut o = self.clone();
        o.use_red_for_error = true;
        o
    }

    #[must_use]
    pub fn svg_link_target(&self) -> Option<&str> {
        self.svg_link_target.as_deref()
    }

    #[must_use]
    pub fn with_svg_link_target(&self, target: impl Into<String>) -> Self {
        let mut o = self.clone();
        o.svg_link_target = Some(target.into());
        o
    }

    #[must_use]
    pub fn preserve_aspect_ratio(&self) -> Option<&str> {
        self.preserve_aspect_ratio.as_deref()
    }

    #[must_use]
    pub fn with_preserve_aspect_ratio(&self, value: impl Into<String>) -> Self {
        let mut o = self.clone();
        o.preserve_aspect_ratio = Some(value.into());
        o
    }

    #[must_use]
    pub fn hover_color(&self) -> Option<&str> {
        self.hover_color.as_deref()
    }

    #[must_use]
    pub fn with_hover_color(&self, color: impl Into<String>) -> Self {
        let mut o = self.clone();
        o.hover_color = Some(color.into());
        o
    }

    #[must_use]
    pub fn watermark(&self) -> Option<&str> {
        self.watermark.as_deref()
    }

    #[must_use]
    pub fn with_watermark(&self, watermark: impl Into<String>) -> Self {
        let mut o = self.clone();
        o.watermark = Some(watermark.into());
        o
    }
}

impl std::fmt::Display for FileFormatOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let o = FileFormatOption::new(FileFormat::Svg);
        assert_eq!(o.file_format(), FileFormat::Svg);
        assert!(o.is_with_metadata());
        assert!((o.scale() - 1.0).abs() < 1e-9);
        assert_eq!(o.decimal(), -1);
    }

    #[test]
    fn builders() {
        let o = FileFormatOption::new(FileFormat::Png)
            .with_scale(2.0)
            .with_decimal(3)
            .with_use_red_for_error();
        assert!((o.scale() - 2.0).abs() < 1e-9);
        assert_eq!(o.decimal(), 3);
        assert!(o.is_use_red_for_error());
    }
}
