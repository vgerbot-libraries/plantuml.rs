//! A single define variable with optional default value.
//!
//! Ported from `net.sourceforge.plantuml.preproc.DefineVariable`.

/// A variable in a `!define` macro signature, optionally with a default value.
///
/// Ported from `net.sourceforge.plantuml.preproc.DefineVariable`.
#[derive(Debug, Clone)]
pub struct DefineVariable {
    name: String,
    default_value: Option<String>,
}

impl DefineVariable {
    /// Creates a new `DefineVariable` by parsing a signature token like `name` or `name="default"`.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DefineVariable.DefineVariable`.
    #[must_use]
    pub fn new(raw: &str) -> Self {
        let name = raw.trim();
        name.find('=').map_or_else(
            || Self {
                name: name.to_string(),
                default_value: None,
            },
            |idx| {
                let var_name = name[..idx].trim().to_string();
                let right = name[idx + 1..].trim();
                // Strip surrounding quotes
                let default = if right.len() >= 2 {
                    right[1..right.len() - 1].to_string()
                } else {
                    right.to_string()
                };
                Self {
                    name: var_name,
                    default_value: Some(default),
                }
            },
        )
    }

    /// Returns the variable name.
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the default value, if any.
    #[must_use]
    pub fn get_default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    /// Creates a new `DefineVariable` with the same name but no default value.
    ///
    /// Ported from `net.sourceforge.plantuml.preproc.DefineVariable.removeDefault`.
    #[must_use]
    pub fn remove_default(&self) -> Self {
        Self {
            name: self.name.clone(),
            default_value: None,
        }
    }
}
