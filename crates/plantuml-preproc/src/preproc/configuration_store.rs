//! Configuration store for `!option` directives.
//!
//! Ported from `net.sourceforge.plantuml.preproc.ConfigurationStore`.

use std::collections::HashMap;
use std::hash::Hash;

/// A key-value store for configuration options.
///
/// Ported from `net.sourceforge.plantuml.preproc.ConfigurationStore`.
#[derive(Debug, Clone)]
pub struct ConfigurationStore<K: Eq + Hash + Clone> {
    values: HashMap<K, String>,
}

impl<K: Eq + Hash + Clone> Default for ConfigurationStore<K> {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash + Clone> ConfigurationStore<K> {
    /// Creates a new empty `ConfigurationStore`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `ConfigurationStore`.
    #[must_use]
    pub fn create_empty() -> Self {
        Self::new()
    }

    /// Defines (sets) a configuration value.
    pub fn define(&mut self, key: K, value: &str) {
        self.values.insert(key, value.to_string());
    }

    /// Returns `true` if the key is defined.
    #[must_use]
    pub fn is_define(&self, key: &K) -> bool {
        self.values.contains_key(key)
    }

    /// Returns `true` if the value for the key is `"true"` (case-insensitive).
    #[must_use]
    pub fn is_true(&self, key: &K) -> bool {
        self.values
            .get(key)
            .is_some_and(|v| v.eq_ignore_ascii_case("true"))
    }

    /// Removes a key.
    pub fn undefine(&mut self, key: &K) {
        self.values.remove(key);
    }

    /// Returns the value for the key, if any.
    #[must_use]
    pub fn get_value(&self, key: &K) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}
