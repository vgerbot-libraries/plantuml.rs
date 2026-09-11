//! Option keys for `!option` directives.
//!
//! Ported from `net.sourceforge.plantuml.preproc.OptionKey`.

/// Keys for `!option` directives in the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.preproc.OptionKey`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptionKey {
    Language,
    UseDescriptiveNames,
    Handwritten,
    Debug,
    SvgDesc,
    SvgTitle,
}

impl OptionKey {
    /// Returns the default value for this option key, if any.
    #[must_use]
    pub fn get_default_value(&self) -> Option<&'static str> {
        match self {
            Self::Handwritten | Self::Debug => Some("true"),
            _ => None,
        }
    }

    /// Simplifies a string by keeping only ASCII letters and lowercasing them.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.OptionKey.simplify`.
    #[must_use]
    pub fn simplify(s: &str) -> String {
        s.chars()
            .filter(|c: &char| c.is_ascii_uppercase() || c.is_ascii_lowercase())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }

    /// Returns the `OptionKey` matching the given string (case-insensitive, ignoring non-letters).
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.OptionKey.lazyFrom`.
    #[must_use]
    pub fn lazy_from(s: &str) -> Option<Self> {
        let simplified = Self::simplify(s);
        [
            Self::Language,
            Self::UseDescriptiveNames,
            Self::Handwritten,
            Self::Debug,
            Self::SvgDesc,
            Self::SvgTitle,
        ]
        .into_iter()
        .find(|key| simplified == Self::simplify(&format!("{key:?}")))
    }
}
