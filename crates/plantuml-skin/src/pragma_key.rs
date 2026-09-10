//! PragmaKey — keys for pragma directives.
//!
//! Ported from: `net/sourceforge/plantuml/skin/PragmaKey.java`

/// Keys for `!pragma` directives recognised by PlantUML.
///
/// Ported from: `net/sourceforge/plantuml/skin/PragmaKey.java`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PragmaKey {
    SequenceMessageSpan,
    EmulateNoGraphvizInstallation,
    EmulateGraphvizCrash,
    EmulateGraphviz244OnWindows,
    Aspect,
    Compact,
    DefaultLabelAngle,
    DefaultLabelDistance,
    EdgeCornerRadius,
    GraphAttributes,
    HorizontalLineBetweenDifferentPackageAllowed,
    Kermor,
    LabelAngle,
    LabelDistance,
    Ratio,
    ShowDeprecation,
    SvgFont,
    SvgInteractive,
    SvgParser,
    SvekTrace,
    Teoz,
    TexSystem,
    TexPreamble,
    UseIntermediatePackages,
    UseVerticalIf,
}

impl PragmaKey {
    /// Returns the default value for this pragma key, if one exists.
    #[must_use]
    pub fn default_value(self) -> Option<&'static str> {
        match self {
            Self::SvekTrace => Some("true"),
            Self::Teoz => Some("true"),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::SequenceMessageSpan => "SEQUENCE_MESSAGE_SPAN",
            Self::EmulateNoGraphvizInstallation => "EMULATE_NO_GRAPHVIZ_INSTALLATION",
            Self::EmulateGraphvizCrash => "EMULATE_GRAPHVIZ_CRASH",
            Self::EmulateGraphviz244OnWindows => "EMULATE_GRAPHVIZ_244_ON_WINDOWS",
            Self::Aspect => "ASPECT",
            Self::Compact => "COMPACT",
            Self::DefaultLabelAngle => "DEFAULT_LABEL_ANGLE",
            Self::DefaultLabelDistance => "DEFAULT_LABEL_DISTANCE",
            Self::EdgeCornerRadius => "EDGE_CORNER_RADIUS",
            Self::GraphAttributes => "GRAPH_ATTRIBUTES",
            Self::HorizontalLineBetweenDifferentPackageAllowed => {
                "HORIZONTAL_LINE_BETWEEN_DIFFERENT_PACKAGE_ALLOWED"
            }
            Self::Kermor => "KERMOR",
            Self::LabelAngle => "LABEL_ANGLE",
            Self::LabelDistance => "LABEL_DISTANCE",
            Self::Ratio => "RATIO",
            Self::ShowDeprecation => "SHOW_DEPRECATION",
            Self::SvgFont => "SVG_FONT",
            Self::SvgInteractive => "SVG_INTERACTIVE",
            Self::SvgParser => "SVG_PARSER",
            Self::SvekTrace => "SVEK_TRACE",
            Self::Teoz => "TEOZ",
            Self::TexSystem => "TEX_SYSTEM",
            Self::TexPreamble => "TEX_PREAMBLE",
            Self::UseIntermediatePackages => "USE_INTERMEDIATE_PACKAGES",
            Self::UseVerticalIf => "USE_VERTICAL_IF",
        }
    }

    /// Case-insensitive, punctuation-insensitive lookup of a pragma key from
    /// a raw string. Returns `None` if no key matches.
    ///
    /// Ported from: `PragmaKey.lazyFrom(String)`.
    #[must_use]
    pub fn lazy_from(s: &str) -> Option<Self> {
        let simplified = simplify(s);
        let all = [
            Self::SequenceMessageSpan,
            Self::EmulateNoGraphvizInstallation,
            Self::EmulateGraphvizCrash,
            Self::EmulateGraphviz244OnWindows,
            Self::Aspect,
            Self::Compact,
            Self::DefaultLabelAngle,
            Self::DefaultLabelDistance,
            Self::EdgeCornerRadius,
            Self::GraphAttributes,
            Self::HorizontalLineBetweenDifferentPackageAllowed,
            Self::Kermor,
            Self::LabelAngle,
            Self::LabelDistance,
            Self::Ratio,
            Self::ShowDeprecation,
            Self::SvgFont,
            Self::SvgInteractive,
            Self::SvgParser,
            Self::SvekTrace,
            Self::Teoz,
            Self::TexSystem,
            Self::TexPreamble,
            Self::UseIntermediatePackages,
            Self::UseVerticalIf,
        ];
        for key in all {
            if simplified == simplify(key.name()) {
                return Some(key);
            }
        }
        None
    }
}

/// Keep only ASCII letters, lowercased.
fn simplify(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_from_finds_keys() {
        assert_eq!(PragmaKey::lazy_from("teoz"), Some(PragmaKey::Teoz));
        assert_eq!(
            PragmaKey::lazy_from("SVG_FONT"),
            Some(PragmaKey::SvgFont)
        );
        assert_eq!(PragmaKey::lazy_from("unknown"), None);
    }

    #[test]
    fn default_values() {
        assert_eq!(PragmaKey::Teoz.default_value(), Some("true"));
        assert_eq!(PragmaKey::Aspect.default_value(), None);
    }
}
