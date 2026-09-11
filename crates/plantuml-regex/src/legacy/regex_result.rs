//! Regex result holding captured groups from a match.
//!
//! Ported from `net.sourceforge.plantuml.regex.RegexResult` (Java).
//!
//! `RegexResult` is a union type: it either holds a map of named
//! `RegexPartialMatch` entries (from the legacy regex engine) or a
//! `UMatcher` (from the ubrex engine). The `get` methods dispatch to
//! whichever backend is active.

use std::collections::HashMap;
use std::fmt;

use crate::ubrex::UMatcher;

use super::regex_partial_match::RegexPartialMatch;

/// Result of a regex match, containing captured groups.
///
/// Ported from `net.sourceforge.plantuml.regex.RegexResult`.
pub enum RegexResult {
    /// Result from the legacy regex engine: a map of named partial matches.
    Data(HashMap<String, RegexPartialMatch>),
    /// Result from the ubrex engine: a `UMatcher` that can look up captures.
    UMatcher(Box<dyn UMatcher>),
}

impl RegexResult {
    /// Creates a result from a map of named partial matches.
    ///
    /// Ported from `RegexResult(Map<String, RegexPartialMatch> data)`.
    pub const fn from_data(data: HashMap<String, RegexPartialMatch>) -> Self {
        Self::Data(data)
    }

    /// Creates a result from a ubrex `UMatcher`.
    ///
    /// Ported from `RegexResult(UMatcher matcher)`.
    pub fn from_umatcher(matcher: Box<dyn UMatcher>) -> Self {
        Self::UMatcher(matcher)
    }

    /// Returns the partial match for the given key, or `None`.
    ///
    /// Ported from `RegexResult.get(String key)`. Throws
    /// `UnsupportedOperationException` in Java when the ubrex matcher is
    /// active; in Rust we return `None`.
    pub fn get(&self, key: &str) -> Option<&RegexPartialMatch> {
        match self {
            Self::Data(data) => data.get(key),
            Self::UMatcher(_) => None,
        }
    }

    /// Returns the captured group at the given index for the named key,
    /// or `None` if not found.
    ///
    /// Ported from `RegexResult.get(String key, int num)`.
    pub fn get_group(&self, key: &str, num: usize) -> Option<String> {
        match self {
            Self::UMatcher(matcher) => {
                let list = matcher.find_values_by_key(key);
                if list.is_empty() {
                    None
                } else {
                    list.get(num).cloned()
                }
            }
            Self::Data(data) => {
                let reg = data.get(key)?;
                reg.get(num).map(std::string::ToString::to_string)
            }
        }
    }

    /// Lazily returns the first non-null captured group at the given index
    /// for any key starting with the given prefix.
    ///
    /// Ported from `RegexResult.getLazzy(String key, int num)`.
    pub fn get_lazzy(&self, key: &str, num: usize) -> Option<String> {
        match self {
            Self::UMatcher(matcher) => {
                let list = matcher.find_first_values_by_key_prefix(key);
                list.and_then(|list| list.get(num).cloned())
            }
            Self::Data(data) => {
                for (k, match_) in data {
                    if !k.starts_with(key) {
                        continue;
                    }
                    if let Some(s) = match_.get(num) {
                        return Some(s.to_string());
                    }
                }
                None
            }
        }
    }

    /// Returns the number of named capture groups.
    ///
    /// Ported from `RegexResult.size()`. Throws
    /// `UnsupportedOperationException` in Java when the ubrex matcher is
    /// active; in Rust we return 0.
    pub fn size(&self) -> usize {
        match self {
            Self::Data(data) => data.len(),
            Self::UMatcher(_) => 0,
        }
    }
}

impl fmt::Display for RegexResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Data(data) => write!(f, "{data:?}"),
            Self::UMatcher(_) => write!(f, "UMatcher"),
        }
    }
}

impl fmt::Debug for RegexResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_data_get() {
        let mut m = RegexPartialMatch::new("key");
        m.add(Some("value".to_string()));
        let mut data = HashMap::new();
        data.insert("key".to_string(), m);
        let result = RegexResult::from_data(data);
        assert_eq!(result.size(), 1);
        let pm = result.get("key");
        assert!(pm.is_some());
        assert_eq!(pm.unwrap().get(0), Some("value"));
        assert_eq!(result.get_group("key", 0), Some("value".to_string()));
        assert!(result.get("missing").is_none());
    }

    #[test]
    fn test_get_lazzy() {
        let mut m1 = RegexPartialMatch::new("foo1");
        m1.add(Some("v1".to_string()));
        let mut m2 = RegexPartialMatch::new("foo2");
        m2.add(Some("v2".to_string()));
        let mut data = HashMap::new();
        data.insert("foo1".to_string(), m1);
        data.insert("foo2".to_string(), m2);
        let result = RegexResult::from_data(data);
        // get_lazzy should find a match starting with "foo"
        let val = result.get_lazzy("foo", 0);
        assert!(val.is_some());
    }
}
