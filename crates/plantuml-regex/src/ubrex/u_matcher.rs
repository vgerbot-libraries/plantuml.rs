/// Matcher interface returned by `UnicodeBracketedExpression::match`.
///
/// Ported from: `com/plantuml/ubrex/UMatcher.java`

use super::capture::Capture;
use super::capture_lookup::CaptureLookup;

pub trait UMatcher: CaptureLookup {
    /// Returns `true` if the pattern matched at the starting position.
    fn start_match(&self) -> bool;

    /// Returns `true` if the pattern matched and consumed the entire text.
    fn exact_match(&self) -> bool;

    /// Returns the text that was matched.
    fn get_accepted_match(&self) -> String;

    /// Returns the first set of values whose key starts with `key_prefix`.
    fn find_first_values_by_key_prefix(&self, key_prefix: &str) -> Option<Vec<String>>;

    /// Returns a capture containing only entries with the given prefix.
    fn extract_by_prefix(&self, key: &str) -> Capture;
}
