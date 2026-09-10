//! Legacy regex trait interface.
//!
//! Ported from `net.sourceforge.plantuml.regex.IRegex` (Java).

use std::collections::HashMap;

use super::regex_partial_match::RegexPartialMatch;
use super::regex_result::RegexResult;

/// Trait for a regex pattern that can match against a string and produce
/// captured groups.
///
/// Ported from `net.sourceforge.plantuml.regex.IRegex`.
pub trait IRegex: Send + Sync {
    /// Returns the regex pattern string (with `%s`, `%q`, `%g` macros unexpanded).
    fn get_pattern_as_string(&self) -> String;

    /// Returns the number of capture groups in this pattern.
    fn count(&self) -> usize;

    /// Creates a map of named partial matches from an iterator of captured
    /// group strings.
    fn create_partial_match(&self, it: &mut dyn Iterator<Item = Option<String>>) -> HashMap<String, RegexPartialMatch>;

    /// Attempts to match the full string. Returns `true` if the pattern
    /// matches the entire input.
    fn r#match(&self, full: &str) -> bool;

    /// Attempts to match the string and returns a `RegexResult` with captured
    /// groups, or `None` if no match.
    fn matcher(&self, full: &str) -> Option<RegexResult>;
}
