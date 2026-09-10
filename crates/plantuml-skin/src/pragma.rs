//! Pragma — pragma directive storage.
//!
//! Ported from: `net/sourceforge/plantuml/skin/Pragma.java`

use std::collections::{HashMap, HashSet};

use crate::pragma_key::PragmaKey;

/// A simple warning record (placeholder for `net.sourceforge.plantuml.warning.Warning`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Warning {
    pub message: String,
}

impl Warning {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Storage for `!pragma` directives.
///
/// Ported from: `net/sourceforge/plantuml/skin/Pragma.java`
#[derive(Debug, Clone)]
pub struct Pragma {
    values: HashMap<PragmaKey, String>,
    warnings: HashSet<Warning>,
}

impl Default for Pragma {
    fn default() -> Self {
        Self::create_empty()
    }
}

impl Pragma {
    /// Creates an empty pragma.
    #[must_use]
    pub fn create_empty() -> Self {
        Self {
            values: HashMap::new(),
            warnings: HashSet::new(),
        }
    }

    /// Defines (or redefines) a pragma value by key name.
    ///
    /// Ported from: `Pragma.define(String, String)`.
    pub fn define(&mut self, key_name: &str, value: Option<String>) {
        if let Some(key) = PragmaKey::lazy_from(key_name) {
            let v = match value {
                Some(v) if v.is_empty() && key.default_value().is_some() => {
                    key.default_value().map(String::from)
                }
                None if key.default_value().is_some() => key.default_value().map(String::from),
                v => v,
            };
            if let Some(v) = v {
                self.values.insert(key, v);
            }
        }
    }

    /// Returns `true` if the given key is defined.
    #[must_use]
    pub fn is_define(&self, key: PragmaKey) -> bool {
        self.values.contains_key(&key)
    }

    /// Removes the given key.
    pub fn undefine(&mut self, key: PragmaKey) {
        self.values.remove(&key);
    }

    /// Returns the value for the given key, if defined.
    #[must_use]
    pub fn get_value(&self, key: PragmaKey) -> Option<&str> {
        self.values.get(&key).map(String::as_str)
    }

    /// Returns `true` if the key's value is `"true"` or `"on"`.
    #[must_use]
    pub fn is_true(&self, key: PragmaKey) -> bool {
        match self.get_value(key) {
            Some(v) => v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("on"),
            None => false,
        }
    }

    /// Returns `true` if the key's value is `"false"` or `"off"`.
    #[must_use]
    pub fn is_false(&self, key: PragmaKey) -> bool {
        match self.get_value(key) {
            Some(v) => v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off"),
            None => false,
        }
    }

    /// Legacy: always returns `true`.
    #[must_use]
    pub fn legacy_replace_backslash_n_by_newline() -> bool {
        true
    }

    /// Adds a warning.
    pub fn add_warning(&mut self, warning: Warning) {
        if !self.warnings.contains(&warning) {
            self.warnings.insert(warning);
        }
    }

    /// Returns all warnings.
    #[must_use]
    pub fn get_warnings(&self) -> &HashSet<Warning> {
        &self.warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_query() {
        let mut p = Pragma::create_empty();
        p.define("teoz", Some("true".into()));
        assert!(p.is_true(PragmaKey::Teoz));
        assert!(!p.is_false(PragmaKey::Teoz));
    }

    #[test]
    fn define_with_default() {
        let mut p = Pragma::create_empty();
        p.define("teoz", None);
        assert!(p.is_true(PragmaKey::Teoz));
    }

    #[test]
    fn undefine() {
        let mut p = Pragma::create_empty();
        p.define("teoz", Some("false".into()));
        assert!(p.is_false(PragmaKey::Teoz));
        p.undefine(PragmaKey::Teoz);
        assert!(!p.is_define(PragmaKey::Teoz));
    }

    #[test]
    fn unknown_key_ignored() {
        let mut p = Pragma::create_empty();
        p.define("nonexistent", Some("value".into()));
        assert!(p.values.is_empty());
    }
}
