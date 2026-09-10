//! Include strategy for `!include` directives.
//!
//! Ported from `net.sourceforge.plantuml.preproc2.PreprocessorIncludeStrategy`.

/// Strategy for handling `!include` directives.
///
/// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorIncludeStrategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessorIncludeStrategy {
    Once,
    Many,
    Default,
}

impl PreprocessorIncludeStrategy {
    /// Parses a strategy from a string.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc2.PreprocessorIncludeStrategy.fromString`.
    #[must_use]
    pub fn from_string(group: &str) -> Self {
        if group.eq_ignore_ascii_case("once") {
            Self::Once
        } else {
            Self::Many
        }
    }
}

impl Default for PreprocessorIncludeStrategy {
    fn default() -> Self {
        Self::Default
    }
}
