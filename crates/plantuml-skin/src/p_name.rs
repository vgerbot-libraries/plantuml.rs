//! PName — style property names.
//!
//! Ported from: `net/sourceforge/plantuml/style/PName.java`

/// Style property names — the CSS-like properties used in PlantUML styles.
///
/// Ported from: `net/sourceforge/plantuml/style/PName.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PName {
    Shadowing,
    FontName,
    FontColor,
    FontSize,
    FontStyle,
    FontWeight,
    BackGroundColor,
    RoundCorner,
    LineThickness,
    DiagonalCorner,
    HyperLinkColor,
    HyperlinkUnderlineStyle,
    HyperlinkUnderlineThickness,
    HeadColor,
    LineColor,
    LineStyle,
    Padding,
    Margin,
    MaximumWidth,
    MinimumWidth,
    ExportedName,
    Image,
    HorizontalAlignment,
    ShowStereotype,
    ImagePosition,
    MarkerShape,
    MarkerSize,
    MarkerColor,
    BarWidth,
    Width,
}

impl PName {
    /// Returns the string name of this property.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Shadowing => "Shadowing",
            Self::FontName => "FontName",
            Self::FontColor => "FontColor",
            Self::FontSize => "FontSize",
            Self::FontStyle => "FontStyle",
            Self::FontWeight => "FontWeight",
            Self::BackGroundColor => "BackGroundColor",
            Self::RoundCorner => "RoundCorner",
            Self::LineThickness => "LineThickness",
            Self::DiagonalCorner => "DiagonalCorner",
            Self::HyperLinkColor => "HyperLinkColor",
            Self::HyperlinkUnderlineStyle => "HyperlinkUnderlineStyle",
            Self::HyperlinkUnderlineThickness => "HyperlinkUnderlineThickness",
            Self::HeadColor => "HeadColor",
            Self::LineColor => "LineColor",
            Self::LineStyle => "LineStyle",
            Self::Padding => "Padding",
            Self::Margin => "Margin",
            Self::MaximumWidth => "MaximumWidth",
            Self::MinimumWidth => "MinimumWidth",
            Self::ExportedName => "ExportedName",
            Self::Image => "Image",
            Self::HorizontalAlignment => "HorizontalAlignment",
            Self::ShowStereotype => "ShowStereotype",
            Self::ImagePosition => "ImagePosition",
            Self::MarkerShape => "MarkerShape",
            Self::MarkerSize => "MarkerSize",
            Self::MarkerColor => "MarkerColor",
            Self::BarWidth => "BarWidth",
            Self::Width => "Width",
        }
    }
}
