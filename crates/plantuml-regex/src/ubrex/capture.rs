//! Named capture group storage.
//!
//! Stores a list of `CaptureEntry` key-value pairs produced by named groups
//! during matching.  The list is immutable — `with_entry`, `merge`, and
//! `with_prefixed_keys` return new `Capture` instances.
//!
//! Ported from: `com/plantuml/ubrex/Capture.java`
use std::fmt::{self, Display, Formatter};

use super::capture_entry::CaptureEntry;
use super::capture_lookup::CaptureLookup;
use super::safe_list::SafeList;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    entries: SafeList<CaptureEntry>,
}

impl Capture {
    /// An empty capture.
    pub const fn empty() -> Self {
        Self {
            entries: SafeList::create_empty(),
        }
    }

    /// Returns a new capture with `(key, value)` appended.
    pub fn with_entry(&self, key: &str, value: &str) -> Self {
        Self {
            entries: self
                .entries
                .add(CaptureEntry::new(key.to_string(), value.to_string())),
        }
    }

    /// Merges `other` into this capture, returning a new `Capture`.
    pub fn merge(&self, other: &Self) -> Self {
        if other.entries.is_empty() {
            return self.clone();
        }
        if self.entries.is_empty() {
            return other.clone();
        }
        Self {
            entries: self.entries.add_all(&other.entries),
        }
    }

    /// Returns a new capture with all keys prefixed by `prefix/`.
    pub fn with_prefixed_keys(&self, prefix: &str) -> Self {
        Self {
            entries: self.entries.mapped(|e| e.with_prefixed_key(prefix)),
        }
    }

    /// Returns all values whose key equals `key`.
    pub fn find_values_by_key(&self, key: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.get_key() == key)
            .map(|e| e.get_value().to_string())
            .collect()
    }

    /// Finds the first entry whose key starts with `key_prefix` and returns all
    /// values for that key, or `None` if no entry matches.
    pub fn find_first_values_by_key_prefix(&self, key_prefix: &str) -> Option<Vec<String>> {
        for entry in self.entries.iter() {
            if entry.get_key().starts_with(key_prefix) {
                return Some(self.find_values_by_key(entry.get_key()));
            }
        }
        None
    }

    /// Returns a new capture containing only entries whose keys start with
    /// `key_prefix`, with the prefix stripped.
    pub fn extract_by_prefix(&self, key_prefix: &str) -> Self {
        let mut result = SafeList::create_empty();
        for entry in self.entries.iter() {
            if let Some(tmp) = entry.without_prefixed_key(key_prefix) {
                result = result.add(tmp);
            }
        }
        Self { entries: result }
    }

    /// Returns the first value matching `key`, or `None`.
    pub fn find_first_value_by_key(&self, key: &str) -> Option<String> {
        for entry in self.entries.iter() {
            if entry.get_key() == key {
                return Some(entry.get_value().to_string());
            }
        }
        None
    }
}

impl CaptureLookup for Capture {
    fn find_first_value_by_key(&self, key: &str) -> Option<String> {
        Self::find_first_value_by_key(self, key)
    }

    fn find_values_by_key(&self, key: &str) -> Vec<String> {
        Self::find_values_by_key(self, key)
    }
}

impl Display for Capture {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.entries, f)
    }
}
