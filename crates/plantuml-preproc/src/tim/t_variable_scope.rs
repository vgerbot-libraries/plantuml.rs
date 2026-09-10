//! Variable scope for preprocessor variables.
//!
//! Ported from `net.sourceforge.plantuml.tim.TVariableScope`.

/// The scope of a variable in the preprocessor.
///
/// Ported from `net.sourceforge.plantuml.tim.TVariableScope`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TVariableScope {
    Local,
    Global,
}

impl TVariableScope {
    /// Parses a scope from a string ("local" or "global").
    ///
    /// Ported from `TVariableScope.lazzyParse`.
    #[must_use]
    pub fn lazzy_parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "local" => Some(Self::Local),
            "global" => Some(Self::Global),
            _ => None,
        }
    }
}
