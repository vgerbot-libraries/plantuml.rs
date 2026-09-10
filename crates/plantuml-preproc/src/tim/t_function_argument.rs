//! Function argument for preprocessor functions.
//!
//! Ported from `net.sourceforge.plantuml.tim.TFunctionArgument`.

use super::expression::TValue;

/// An argument of a preprocessor function, with an optional default value.
///
/// Ported from `net.sourceforge.plantuml.tim.TFunctionArgument`.
#[derive(Debug, Clone)]
pub struct TFunctionArgument {
    name: String,
    def: Option<TValue>,
}

impl TFunctionArgument {
    /// Creates a new `TFunctionArgument`.
    #[must_use]
    pub fn new(name: impl Into<String>, def: Option<TValue>) -> Self {
        Self {
            name: name.into(),
            def,
        }
    }

    /// Returns the argument name.
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the optional default value.
    #[must_use]
    pub fn get_optional_default_value(&self) -> Option<&TValue> {
        self.def.as_ref()
    }
}

impl std::fmt::Display for TFunctionArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ARG:{}", self.name)
    }
}
