//! A single key-value capture entry produced by a named group match.
//!
//! Ported from: `com/plantuml/ubrex/CaptureEntry.java`
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureEntry {
    key: String,
    value: String,
}

impl CaptureEntry {
    pub const fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Returns a new entry with `prefix` prepended to the key, separated by `/`.
    pub fn with_prefixed_key(&self, prefix: &str) -> Self {
        Self {
            key: format!("{prefix}/{}", self.key),
            value: self.value.clone(),
        }
    }

    /// Returns a new entry with `prefix` stripped from the key, or `None` if
    /// the key does not start with `prefix/`.
    pub fn without_prefixed_key(&self, prefix: &str) -> Option<Self> {
        let needle = format!("{prefix}/");
        if self.key.starts_with(&needle) {
            Some(Self {
                key: self.key[prefix.len() + 1..].to_string(),
                value: self.value.clone(),
            })
        } else {
            None
        }
    }
}

impl Display for CaptureEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}
