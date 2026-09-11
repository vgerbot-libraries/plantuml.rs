//! Trait for looking up captured values by key.
//!
//! Ported from: `com/plantuml/ubrex/CaptureLookup.java`
pub trait CaptureLookup {
    /// Returns the first value matching `key`, or `None`.
    fn find_first_value_by_key(&self, key: &str) -> Option<String>;

    /// Returns all values matching `key`.
    fn find_values_by_key(&self, key: &str) -> Vec<String>;
}
